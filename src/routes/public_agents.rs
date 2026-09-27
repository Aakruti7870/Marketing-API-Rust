use crate::error::AppError;
use crate::models::PublicAgentChatDto;
use crate::services::agent_service;
use crate::state::AppState;
use axum::{extract::{Path, State}, http::header, response::{IntoResponse, Response}, routing::{get, post}, Json, Router};
use crate::utils::response::json_success;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/:public_key/chat", post(chat))
        .route("/:public_key/widget.js", get(widget))
        .with_state(state)
}

async fn chat(
    State(state): State<AppState>,
    Path(public_key): Path<String>,
    Json(dto): Json<PublicAgentChatDto>,
) -> Result<impl IntoResponse, AppError> {
    if dto.message.trim().is_empty() {
        return Err(AppError::Validation("message is required".into()));
    }
    if dto.message.len() > 4000 {
        return Err(AppError::Validation("message is too long".into()));
    }
    Ok(json_success(
        agent_service::public_chat(&state.pool, &state.config, &public_key, dto).await?,
        "AI agent response generated",
    ))
}


async fn widget(Path(public_key): Path<String>) -> Response {
    let key=public_key.replace('\\', "").replace('"', "");
    let script=format!(r#"
(() => {{
  const KEY = "{key}";
  const API = "https://api.goldetech.com/api/public/agents/" + KEY + "/chat";
  const ID = "golde-ai-widget";
  if (document.getElementById(ID)) return;
  const style=document.createElement("style");
  style.textContent=\`
    #golde-ai-widget{{position:fixed;right:22px;bottom:22px;z-index:2147483647;font-family:Inter,system-ui,sans-serif}}
    #golde-ai-widget button{{font:inherit}}
    .ge-bubble{{width:58px;height:58px;border:0;border-radius:50%;background:linear-gradient(135deg,#6e49d8,#d95f9f);color:#fff;box-shadow:0 12px 35px rgba(67,38,120,.28);cursor:pointer;font-size:22px}}
    .ge-panel{{display:none;width:350px;height:500px;margin-bottom:12px;border:1px solid #e8e0f2;border-radius:18px;background:#fff;box-shadow:0 20px 60px rgba(33,20,55,.2);overflow:hidden}}
    .ge-panel.open{{display:flex;flex-direction:column}}
    .ge-head{{padding:16px;background:linear-gradient(135deg,#f8f5ff,#fff);border-bottom:1px solid #eee8f5}}
    .ge-head strong{{display:block;color:#251c31}} .ge-head small{{color:#81768d}}
    .ge-msgs{{flex:1;padding:14px;overflow:auto;background:#fcfbfe}}
    .ge-msg{{max-width:82%;padding:10px 12px;margin:7px 0;border-radius:13px;font-size:13px;line-height:1.45;white-space:pre-wrap}}
    .ge-user{{margin-left:auto;background:#eee8fb;color:#2f2340}} .ge-ai{{background:#fff;border:1px solid #eee8f4;color:#352c3e}}
    .ge-input{{display:flex;gap:8px;padding:10px;border-top:1px solid #eee8f4}} .ge-input input{{flex:1;border:1px solid #ddd4e8;border-radius:10px;padding:10px;outline:none}} .ge-input button{{border:0;border-radius:10px;padding:0 14px;background:#6e49d8;color:#fff;cursor:pointer}}
    @media(max-width:480px){{.ge-panel{{width:calc(100vw - 28px);height:70vh}} #golde-ai-widget{{right:14px;bottom:14px}}}}
  \`;
  document.head.appendChild(style);
  const root=document.createElement("div");root.id=ID;
  root.innerHTML=\`<div class="ge-panel"><div class="ge-head"><strong>AI Assistant</strong><small>Usually replies in seconds</small></div><div class="ge-msgs"></div><div class="ge-input"><input placeholder="Type your message…"/><button>Send</button></div></div><button class="ge-bubble" aria-label="Open chat">✦</button>\`;
  document.body.appendChild(root);
  const panel=root.querySelector(".ge-panel"), bubble=root.querySelector(".ge-bubble"), input=root.querySelector("input"), send=root.querySelector(".ge-input button"), msgs=root.querySelector(".ge-msgs");
  const cidKey="golde_agent_conversation_"+KEY; let cid=localStorage.getItem(cidKey);
  const add=(text,who)=>{{const el=document.createElement("div");el.className="ge-msg "+(who==="user"?"ge-user":"ge-ai");el.textContent=text;msgs.appendChild(el);msgs.scrollTop=msgs.scrollHeight;}};
  const ask=async()=>{{const text=input.value.trim();if(!text)return;input.value="";add(text,"user");send.disabled=true;try{{const res=await fetch(API,{{method:"POST",headers:{{"Content-Type":"application/json"}},body:JSON.stringify({{conversation_id:cid,visitor_id:localStorage.getItem("golde_visitor_id")||crypto.randomUUID(),message:text}})}});const body=await res.json();const data=body.data||body;if(data.conversation_id){{cid=data.conversation_id;localStorage.setItem(cidKey,cid)}}add(data.reply||"I’m sorry, I couldn’t respond right now.","ai");}}catch(e){{add("I’m having trouble connecting. Please try again or contact the business.","ai")}}finally{{send.disabled=false;input.focus()}}}};
  bubble.onclick=()=>{{panel.classList.toggle("open");if(panel.classList.contains("open"))input.focus()}};
  send.onclick=ask;input.onkeydown=e=>{{if(e.key==="Enter")ask()}};
}})();
"#);
    ([(header::CONTENT_TYPE,"application/javascript; charset=utf-8"),(header::CACHE_CONTROL,"public, max-age=300")],script).into_response()
}
