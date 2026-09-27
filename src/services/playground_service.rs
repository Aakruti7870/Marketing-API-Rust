use crate::config::Config;
use crate::error::AppError;
use crate::models::AiAgentChannelConnection;
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use axum::extract::Multipart;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use csv::ReaderBuilder;
use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

fn decrypt_channel_secret(config: &Config, value: &str) -> Result<String, AppError> {
    let master = config.channel_encryption_key.as_deref()
        .ok_or_else(|| AppError::BadRequest("CHANNEL_ENCRYPTION_KEY must be configured before using customer channels.".into()))?;
    let digest = Sha256::digest(master.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    let packed = B64.decode(value).map_err(|_| AppError::ExternalService("Invalid stored channel credential".into()))?;
    if packed.len() < 13 { return Err(AppError::ExternalService("Invalid stored channel credential".into())); }
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::BadRequest("Invalid channel encryption key".into()))?;
    let nonce = Nonce::from_slice(&packed[..12]);
    let plaintext = cipher.decrypt(nonce, &packed[12..])
        .map_err(|_| AppError::ExternalService("Unable to decrypt channel credential".into()))?;
    String::from_utf8(plaintext).map_err(|_| AppError::ExternalService("Invalid channel credential encoding".into()))
}

fn validate_kind(kind: &str) -> Result<String, AppError> {
    let value = kind.trim().to_uppercase();
    if value == "VIDEO" || value == "VIDEO_GENERATION" {
        return Err(AppError::BadRequest("Video generation is intentionally disabled in GOLD-e Playground.".into()));
    }
    let allowed = ["BANNER", "OFFER_IMAGE", "LOGO", "EMAIL_TEMPLATE", "WHATSAPP_TEMPLATE", "SMART_WHATSAPP", "SEO"];
    if !allowed.contains(&value.as_str()) {
        return Err(AppError::Validation("Unsupported Playground type.".into()));
    }
    Ok(value)
}

fn system_prompt(kind: &str) -> &'static str {
    match kind {
        "EMAIL_TEMPLATE" => "Create a production-ready business email template. Return subject, preheader, body copy, CTA and a concise plain-text version. Do not invent claims, prices or guarantees.",
        "WHATSAPP_TEMPLATE" => "Create a concise WhatsApp business message template with a clear purpose, variables like {{name}} when useful, and a compliant call to action. Avoid spammy claims.",
        "SMART_WHATSAPP" => "Create a smart WhatsApp customer message. Return a short message plus 2-4 quick-reply button labels and machine-readable button payloads that can map to GOLD-e bot intents or automations. Keep it natural and useful.",
        "SEO" => "Act as an SEO strategist. Produce a structured SEO package: primary keyword, secondary keywords, title tag, meta description, URL slug, H1, supporting headings, FAQ questions, internal-link suggestions, Open Graph title/description and JSON-LD type. Never promise rankings.",
        _ => "Create polished marketing copy for the requested business asset. Keep claims grounded in the supplied business context and optimize for clarity, conversion and brand consistency.",
    }
}

async fn chat_generate(config: &Config, kind: &str, prompt: &str, context: &str) -> Result<String, AppError> {
    let api_key = config.ai_api_key.clone().ok_or_else(|| AppError::BadRequest("AI provider is not configured. Set AI_API_KEY on the backend.".into()))?;
    let body = json!({
        "model": config.ai_model,
        "messages": [
            {"role":"system","content":system_prompt(kind)},
            {"role":"user","content":format!("Business context:\n{}\n\nRequest:\n{}", context, prompt)}
        ],
        "temperature": 0.35
    });
    let response = Client::new()
        .post(format!("{}/chat/completions", config.ai_api_base_url.trim_end_matches('/')))
        .bearer_auth(api_key)
        .json(&body)
        .send().await
        .map_err(|e| AppError::ExternalService(format!("Playground AI request failed: {}", e)))?;
    let status = response.status();
    let raw = response.text().await.map_err(|e| AppError::ExternalService(format!("Unable to read AI response: {}", e)))?;
    let payload: Value = serde_json::from_str(&raw).map_err(|_| AppError::ExternalService(format!("Invalid AI response (HTTP {})", status)))?;
    if !status.is_success() {
        let msg = payload.get("error").and_then(|e| e.get("message")).and_then(Value::as_str).unwrap_or("AI provider error");
        return Err(AppError::ExternalService(format!("HTTP {}: {}", status, msg)));
    }
    payload.get("choices").and_then(Value::as_array).and_then(|a| a.first())
        .and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::ExternalService("AI provider returned no text output".into()))
}

pub async fn generate_text(
    config: &Config, pool: &PgPool, workspace_id: Uuid, user_id: Uuid,
    kind: &str, prompt: &str, context: &str,
) -> Result<Value, AppError> {
    let kind = validate_kind(kind)?;
    let content = chat_generate(config, &kind, prompt, context).await?;
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO playground_assets (id,workspace_id,kind,title,prompt,content,metadata,created_by_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"
    )
    .bind(id).bind(workspace_id).bind(&kind).bind(format!("{} draft", kind.replace('_', " ")))
    .bind(prompt).bind(&content).bind(json!({})).bind(user_id)
    .execute(pool).await?;
    Ok(json!({"id":id,"kind":kind,"content":content}))
}

pub async fn generate_image(
    config: &Config, pool: &PgPool, workspace_id: Uuid, user_id: Uuid,
    kind: &str, prompt: &str, title: &str,
) -> Result<Value, AppError> {
    let kind = validate_kind(kind)?;
    if !matches!(kind.as_str(), "BANNER" | "OFFER_IMAGE" | "LOGO") {
        return Err(AppError::Validation("Image generation is available only for Banner, Offer Picture and Logo.".into()));
    }
    let api_key = config.ai_api_key.clone().ok_or_else(|| AppError::BadRequest("AI provider is not configured. Set AI_API_KEY on the backend.".into()))?;
    let body = json!({
        "model": config.ai_image_model,
        "prompt": prompt,
        "size": "1024x1024",
        "response_format": "b64_json"
    });
    let response = Client::new()
        .post(format!("{}/images/generations", config.ai_api_base_url.trim_end_matches('/')))
        .bearer_auth(api_key)
        .json(&body)
        .send().await
        .map_err(|e| AppError::ExternalService(format!("Image provider request failed: {}", e)))?;
    let status = response.status();
    let payload: Value = response.json().await.map_err(|e| AppError::ExternalService(format!("Invalid image provider response: {}", e)))?;
    if !status.is_success() {
        let msg = payload.get("error").and_then(|e| e.get("message")).and_then(Value::as_str).unwrap_or("Image provider error");
        return Err(AppError::ExternalService(format!("HTTP {}: {}", status, msg)));
    }

    let item = payload.get("data").and_then(Value::as_array).and_then(|a| a.first()).ok_or_else(|| AppError::ExternalService("Image provider returned no image".into()))?;
    let (bytes, mime) = if let Some(b64) = item.get("b64_json").and_then(Value::as_str) {
        (B64.decode(b64).map_err(|_| AppError::ExternalService("Invalid base64 image returned by provider".into()))?, "image/png".to_string())
    } else if let Some(url) = item.get("url").and_then(Value::as_str) {
        let img = Client::new().get(url).send().await.map_err(|e| AppError::ExternalService(format!("Unable to fetch generated image: {}", e)))?;
        let mime = img.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("image/png").to_string();
        (img.bytes().await.map_err(|e| AppError::ExternalService(format!("Unable to read generated image: {}", e)))?.to_vec(), mime)
    } else {
        return Err(AppError::ExternalService("Image provider returned neither b64_json nor url".into()));
    };

    let id = Uuid::new_v4();
    let public_key = Uuid::new_v4().to_string().replace('-', "");
    sqlx::query(
        "INSERT INTO playground_assets (id,workspace_id,kind,title,prompt,media_data,mime_type,public_key,metadata,created_by_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
    )
    .bind(id).bind(workspace_id).bind(&kind).bind(title).bind(prompt).bind(bytes).bind(mime).bind(&public_key)
    .bind(json!({"video_generation":false})).bind(user_id).execute(pool).await?;

    let public_url = format!("{}/api/public/playground/assets/{}", config.public_base_url.trim_end_matches('/'), public_key);
    Ok(json!({"id":id,"kind":kind,"title":title,"public_url":public_url}))
}

pub async fn list_groups(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT g.id,g.name,COUNT(gm.contact_id)::BIGINT AS contact_count
         FROM contact_groups g LEFT JOIN contact_group_members gm ON gm.group_id=g.id
         WHERE g.workspace_id=$1 GROUP BY g.id,g.name ORDER BY g.updated_at DESC"
    ).bind(workspace_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| json!({
        "id": row.get::<Uuid,_>("id"),
        "name": row.get::<String,_>("name"),
        "contact_count": row.get::<i64,_>("contact_count")
    })).collect())
}

pub async fn list_assets(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id,kind,title,content,mime_type,public_key,created_at FROM playground_assets
         WHERE workspace_id=$1 ORDER BY created_at DESC LIMIT 100"
    ).bind(workspace_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| {
        let id:Uuid=row.get("id"); let kind:String=row.get("kind"); let title:String=row.get("title");
        let content:Option<String>=row.get("content"); let mime:Option<String>=row.get("mime_type");
        let public_key:Option<String>=row.get("public_key"); let created_at:chrono::DateTime<chrono::Utc>=row.get("created_at");
        json!({"id":id,"kind":kind,"title":title,"content":content,"mime_type":mime,"public_key":public_key,"created_at":created_at})
    }).collect())
}

pub async fn import_contacts(
    pool: &PgPool, workspace_id: Uuid, user_id: Uuid, channel_id: Uuid, mut multipart: Multipart
) -> Result<Value, AppError> {
    let mut file_bytes:Option<Vec<u8>>=None;
    let mut file_name="contacts.csv".to_string();
    let mut group_name:Option<String>=None;

    while let Some(field)=multipart.next_field().await.map_err(|e| AppError::BadRequest(format!("Invalid upload: {}",e)))? {
        let name=field.name().unwrap_or("");
        if name=="file" {
            if let Some(n)=field.file_name(){file_name=n.to_string();}
            file_bytes=Some(field.bytes().await.map_err(|e| AppError::BadRequest(format!("Unable to read contact file: {}",e)))?.to_vec());
        } else if name=="group_name" {
            group_name=Some(field.text().await.map_err(|e| AppError::BadRequest(format!("Unable to read group name: {}",e)))?);
        }
    }

    let bytes=file_bytes.ok_or_else(|| AppError::Validation("Contact import file is required.".into()))?;
    if bytes.len()>10*1024*1024 { return Err(AppError::BadRequest("Contact import file must be 10 MB or smaller.".into())); }
    let group_name=group_name.filter(|s|!s.trim().is_empty()).unwrap_or_else(|| {
        file_name.rsplit_once('.').map(|(n,_)|n).unwrap_or(&file_name).trim().to_string()
    });
    if group_name.is_empty(){return Err(AppError::Validation("Contact group name is required.".into()));}

    let channel=sqlx::query_as::<_,AiAgentChannelConnection>(
        "SELECT * FROM ai_agent_channel_connections WHERE id=$1 AND workspace_id=$2"
    ).bind(channel_id).bind(workspace_id).fetch_optional(pool).await?
     .ok_or_else(|| AppError::NotFound("Channel connection not found.".into()))?;

    if channel.channel!="WHATSAPP" {
        return Err(AppError::BadRequest("Direct contact-group import is currently enabled for WhatsApp channels.".into()));
    }

    let mut reader=ReaderBuilder::new().flexible(true).trim(csv::Trim::All).from_reader(bytes.as_slice());
    let headers=reader.headers().map_err(|e|AppError::Validation(format!("Invalid CSV headers: {}",e)))?.clone();
    let find=|names:&[&str]| headers.iter().position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)));
    let phone_i=find(&["phone","mobile","whatsapp","phone_number"]).ok_or_else(||AppError::Validation("CSV must contain a phone/mobile/whatsapp column.".into()))?;
    let first_i=find(&["first_name","first name","name","full_name"]);
    let last_i=find(&["last_name","last name"]);
    let email_i=find(&["email","email_address"]);
    let company_i=find(&["company","company_name"]);

    let group_id=sqlx::query_scalar::<_,Uuid>(
        "INSERT INTO contact_groups (workspace_id,name,description,created_by_id)
         VALUES ($1,$2,$3,$4)
         ON CONFLICT(workspace_id,name) DO UPDATE SET updated_at=NOW()
         RETURNING id"
    ).bind(workspace_id).bind(&group_name).bind(format!("Imported from {} for {}",file_name,channel.display_name.clone().unwrap_or_else(||"WhatsApp".into()))).bind(user_id)
    .fetch_one(pool).await?;

    let mut imported=0u64; let mut skipped=0u64;
    for record in reader.records() {
        let record=record.map_err(|e|AppError::Validation(format!("Invalid CSV row: {}",e)))?;
        let phone=record.get(phone_i).unwrap_or("").trim();
        if phone.is_empty(){skipped+=1;continue;}
        let first=first_i.and_then(|i|record.get(i)).unwrap_or("Customer").trim();
        let first=if first.is_empty(){"Customer"}else{first};
        let last=last_i.and_then(|i|record.get(i)).unwrap_or("").trim();
        let email=email_i.and_then(|i|record.get(i)).unwrap_or("").trim();
        let company=company_i.and_then(|i|record.get(i)).unwrap_or("").trim();

        let existing=sqlx::query_scalar::<_,Uuid>(
            "SELECT id FROM contacts WHERE workspace_id=$1 AND regexp_replace(phone,'\\D','','g')=regexp_replace($2,'\\D','','g') LIMIT 1"
        ).bind(workspace_id).bind(phone).fetch_optional(pool).await?;
        let contact_id=if let Some(id)=existing{id}else{
            sqlx::query_scalar::<_,Uuid>(
                "INSERT INTO contacts (workspace_id,first_name,last_name,email,phone,company,status)
                 VALUES ($1,$2,$3,$4,$5,$6,'ACTIVE') RETURNING id"
            ).bind(workspace_id).bind(first).bind(if last.is_empty(){None}else{Some(last)})
             .bind(if email.is_empty(){None}else{Some(email)}).bind(phone)
             .bind(if company.is_empty(){None}else{Some(company)}).fetch_one(pool).await?
        };
        sqlx::query("INSERT INTO contact_group_members(group_id,contact_id) VALUES($1,$2) ON CONFLICT DO NOTHING")
            .bind(group_id).bind(contact_id).execute(pool).await?;
        imported+=1;
    }

    sqlx::query("INSERT INTO channel_contact_groups(channel_id,group_id) VALUES($1,$2) ON CONFLICT DO NOTHING")
        .bind(channel_id).bind(group_id).execute(pool).await?;

    Ok(json!({"channel_id":channel_id,"group_id":group_id,"group_name":group_name,"imported":imported,"skipped":skipped}))
}

pub async fn share_whatsapp(
    config:&Config, pool:&PgPool, workspace_id:Uuid, channel_id:Uuid, group_id:Uuid, asset_id:Uuid, caption:Option<&str>
) -> Result<Value,AppError>{
    let channel=sqlx::query_as::<_,AiAgentChannelConnection>(
        "SELECT * FROM ai_agent_channel_connections WHERE id=$1 AND workspace_id=$2 AND status='ACTIVE'"
    ).bind(channel_id).bind(workspace_id).fetch_optional(pool).await?
     .ok_or_else(||AppError::NotFound("Active WhatsApp channel not found.".into()))?;
    if channel.channel!="WHATSAPP"{return Err(AppError::BadRequest("Selected channel is not WhatsApp.".into()));}
    let secret=channel.secret_ciphertext.as_deref().ok_or_else(||AppError::BadRequest("WhatsApp channel access token is not configured.".into()))?;
    let token=decrypt_channel_secret(config,secret)?;
    let sender_id=channel.external_account_id.as_deref().or(channel.external_sender_id.as_deref()).ok_or_else(||AppError::BadRequest("WhatsApp phone number/account ID is not configured.".into()))?;

    let asset=sqlx::query(
        "SELECT kind,content,media_data,mime_type,public_key FROM playground_assets WHERE id=$1 AND workspace_id=$2"
    ).bind(asset_id).fetch_optional(pool).await?.ok_or_else(||AppError::NotFound("Playground asset not found.".into()))?;
    let kind:String=asset.get("kind"); let content:Option<String>=asset.get("content"); let media_data:Option<Vec<u8>>=asset.get("media_data"); let public_key:Option<String>=asset.get("public_key");
    let recipients=sqlx::query("SELECT c.id,c.phone FROM contact_group_members gm JOIN contacts c ON c.id=gm.contact_id WHERE gm.group_id=$1 AND c.workspace_id=$2")
        .bind(group_id).bind(workspace_id).fetch_all(pool).await?;
    if recipients.is_empty(){return Ok(json!({"sent":0,"failed":0,"message":"The selected contact group is empty."}));}
    let client=Client::new();
    let mut sent=0u64; let mut failed=0u64; let mut errors=Vec::new();
    for row in recipients{
        let phone:String=row.get("phone");
        let payload=if media_data.is_some()&&public_key.is_some(){
            let url=format!("{}/api/public/playground/assets/{}",config.public_base_url.trim_end_matches('/'),public_key.as_ref().unwrap());
            json!({"messaging_product":"whatsapp","to":phone,"type":"image","image":{"link":url,"caption":caption.unwrap_or("")}})
        }else{
            json!({"messaging_product":"whatsapp","to":phone,"type":"text","text":{"preview_url":false,"body":caption.or(content.as_deref()).unwrap_or("")}})
        };
        let response=client.post(format!("https://graph.facebook.com/{}/messages",config.whatsapp_api_version)).bearer_auth(token.clone()).json(&payload).send().await;
        match response{
            Ok(resp) if resp.status().is_success()=>sent+=1,
            Ok(resp)=>{failed+=1;let body=resp.text().await.unwrap_or_default();if errors.len()<5{errors.push(body)}},
            Err(e)=>{failed+=1;if errors.len()<5{errors.push(e.to_string())}},
        }
    }
    Ok(json!({"sent":sent,"failed":failed,"errors":errors,"group_id":group_id,"asset_id":asset_id,"sender_id":sender_id}))
}
