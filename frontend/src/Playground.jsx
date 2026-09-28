import { useEffect, useMemo, useState } from "react";
import {
  Bot, Image, Mail, MessageSquare, Search, Sparkles, Send, Copy, Check,
  Share2, RefreshCw, Wand2, FileText, Upload
} from "lucide-react";
import { agentsApi, playgroundApi, automationsApi, unwrap } from "./services/api";
import "./Playground.css";

const TABS = [
  ["chat","Chat Playground",Bot],
  ["images","Images",Image],
  ["templates","Templates",FileText],
  ["seo","SEO",Search],
];
const IMAGE_TYPES = [["BANNER","Business Banner"],["OFFER_IMAGE","Offer Picture"],["LOGO","Logo"]];
const TEXT_TYPES = [["EMAIL_TEMPLATE","Email Template"],["WHATSAPP_TEMPLATE","WhatsApp Message Template"],["SMART_WHATSAPP","Smart WhatsApp Message"]];


function SmartButtonEditor({asset,actions,automations,onSave}){
  const buttons=asset?.metadata?.buttons||[];
  if(!buttons.length)return null;
  return <div className="smart-button-editor">
    <span className="smart-editor-title">REAL WHATSAPP QUICK REPLIES</span>
    {buttons.map(button=>{
      const current=actions.find(a=>a.button_id===button.id);
      const type=current?.action_type||"BOT_REPLY";
      const automationId=current?.action_payload?.automation_id||"";
      return <div className="smart-action-row" key={button.id}>
        <button className="smart-preview-button">{button.title}</button>
        <select value={type} onChange={e=>onSave({...button,assetId:asset.id},e.target.value,automationId)}>
          <option value="BOT_REPLY">Feed to AI Bot</option>
          <option value="AUTOMATION">Trigger Automation</option>
        </select>
        {type==="AUTOMATION"&&<select value={automationId} onChange={e=>onSave({...button,assetId:asset.id},type,e.target.value)}>
          <option value="">Select automation</option>
          {automations.map(a=><option key={a.id} value={a.id}>{a.name}</option>)}
        </select>}
        {type==="BOT_REPLY"&&<small>Payload: {button.payload}</small>}
      </div>;
    })}
    <small className="smart-action-help">Choose what happens when a customer taps each WhatsApp button. The assignment is stored and used by the webhook.</small>
  </div>;
}

function Playground() {
  const [tab,setTab]=useState("chat");
  const [mode,setMode]=useState("SMART_WHATSAPP");
  const [prompt,setPrompt]=useState("");
  const [context,setContext]=useState("");
  const [chat,setChat]=useState([]);
  const [busy,setBusy]=useState(false);
  const [notice,setNotice]=useState("");
  const [error,setError]=useState("");
  const [assets,setAssets]=useState([]);
  const [groups,setGroups]=useState([]);
  const [channels,setChannels]=useState([]);
  const [automations,setAutomations]=useState([]);
  const [selectedAgentId,setSelectedAgentId]=useState("");
  const [smartActions,setSmartActions]=useState([]);
  const [selectedChannel,setSelectedChannel]=useState("");
  const [selectedGroup,setSelectedGroup]=useState("");
  const [selectedAsset,setSelectedAsset]=useState("");
  const [caption,setCaption]=useState("");
  const [copied,setCopied]=useState(false);

  const selectedAssetRow=useMemo(()=>assets.find(a=>a.id===selectedAsset),[assets,selectedAsset]);
  const imageAssets=assets.filter(a=>["BANNER","OFFER_IMAGE","LOGO"].includes(a.kind) && a.public_key);

  const loadData=async()=>{
    try{
      const [g,x,a,au]=await Promise.all([playgroundApi.groups(),playgroundApi.assets(),agentsApi.list(),automationsApi.list()]);
      setGroups(unwrap(g)||[]);
      setAssets(unwrap(x)||[]);
      const agentList=unwrap(a)||[];
      setAutomations(unwrap(au)||[]);
      const active=agentList.find(v=>v.status==="ACTIVE");
      setSelectedAgentId(prev=>prev||active?.id||"");
      if(active){
        const cs=unwrap(await agentsApi.channels(active.id))||[];
        const wa=cs.filter(c=>c.channel==="WHATSAPP" && c.status==="ACTIVE");
        setChannels(wa);
        setSelectedChannel(prev=>prev||wa[0]?.id||"");
      } else setChannels([]);
    }catch(e){setError(e?.response?.data?.error||e?.message||"Could not load Playground resources.");}
  };
  useEffect(()=>{loadData()},[]);

  const loadButtonActions=async(assetId)=>{
    if(!assetId)return;
    try{setSmartActions(unwrap(await playgroundApi.buttonActions(assetId))||[]);}catch(e){setError(e?.response?.data?.error||e?.message||"Could not load button actions.");}
  };

  const runText=async(kind=mode)=>{
    if(!prompt.trim()||busy)return;
    const request=prompt.trim();
    setPrompt("");setBusy(true);setError("");setNotice("");
    setChat(prev=>[...prev,{role:"user",content:request}]);
    try{
      const data=unwrap(await playgroundApi.generate({kind,prompt:request,business_context:context}));
      setChat(prev=>[...prev,{role:"assistant",content:data?.content||"",asset:data}]);
      if(kind==="SMART_WHATSAPP" && data?.id) await loadButtonActions(data.id);
      await loadData();
    }catch(e){
      const message=e?.response?.data?.error||e?.message||"Generation failed.";
      setChat(prev=>[...prev,{role:"error",content:message}]);setError(message);
    }finally{setBusy(false);}
  };

  const generateImage=async(kind)=>{
    if(!prompt.trim()||busy)return;
    setBusy(true);setError("");setNotice("");
    try{
      const data=unwrap(await playgroundApi.generateImage({kind,prompt:prompt.trim(),title:kind.replaceAll("_"," ")+" — GOLD-e Playground"}));
      setSelectedAsset(data?.id||"");setPrompt("");
      setNotice("Image created and saved to your Playground library.");
      await loadData();
    }catch(e){setError(e?.response?.data?.error||e?.message||"Image generation failed.");}
    finally{setBusy(false);}
  };


  const saveAction=async(button, actionType, automationId="")=>{
    if(!selectedAgentId)return setError("Select or deploy an active AI Agent before assigning button actions.");
    try{
      const payload=actionType==="AUTOMATION"?{automation_id:automationId}:{message:button.payload||button.title};
      await playgroundApi.saveButtonAction({button_id:button.id,asset_id:button.assetId,agent_id:selectedAgentId,title:button.title,action_type:actionType,action_payload:payload});
      await loadButtonActions(button.assetId); setNotice("Button action saved.");
    }catch(e){setError(e?.response?.data?.error||e?.message||"Could not save button action.");}
  };

  const copyText=async(text)=>{
    try{await navigator.clipboard.writeText(text);setCopied(true);setTimeout(()=>setCopied(false),1500)}catch{}
  };

  const share=async()=>{
    if(!selectedChannel||!selectedGroup||!selectedAsset){setError("Select a WhatsApp channel, contact group and generated image first.");return;}
    setBusy(true);setError("");setNotice("");
    try{
      const data=unwrap(await playgroundApi.shareWhatsApp({channel_id:selectedChannel,group_id:selectedGroup,asset_id:selectedAsset,caption:caption||undefined}));
      setNotice("WhatsApp distribution complete: "+(data.sent||0)+" sent, "+(data.failed||0)+" failed.");
    }catch(e){setError(e?.response?.data?.error||e?.message||"WhatsApp distribution failed.");}
    finally{setBusy(false);}
  };

  return <div className="playground-page">
    <div className="playground-shell">
      <aside className="playground-rail">
        <div className="playground-brand"><Sparkles size={17}/><span>GOLD-e Playground</span></div>
        <div className="playground-mode-note">Create marketing assets, customer messages and SEO packages. <b>Video generation is not available.</b></div>
        <div className="playground-tabs">
          {TABS.map(([id,label,Icon])=><button key={id} className={tab===id?"active":""} onClick={()=>{setTab(id);setError("");}}><Icon size={16}/><span>{label}</span></button>)}
        </div>
        <div className="playground-mini-list">
          <span>RECENT ASSETS</span>
          {assets.slice(0,7).map(a=><button key={a.id} onClick={()=>{setSelectedAsset(a.id);setTab(a.public_key?"images":"templates");}}><small>{a.kind.replaceAll("_"," ")}</small><strong>{a.title}</strong></button>)}
          {!assets.length&&<em>No generated assets yet.</em>}
        </div>
      </aside>

      <main className="playground-main">
        <header className="playground-top">
          <div><span className="panel-kicker">CREATIVE WORKSPACE</span><h1>What are you building?</h1><p>Chat with GOLD-e like a creative copilot, then turn the result into a reusable business asset.</p></div>
          <button className="ghost-btn compact" onClick={loadData}><RefreshCw size={14}/>Sync library</button>
        </header>

        {(error||notice)&&<div className={"alert "+(error?"error":"success")}>{error||notice}</div>}

        {tab==="chat"&&<section className="playground-chat-layout">
          <div className="playground-chat panel">
            <div className="playground-chat-head"><div><span className="panel-kicker">CHATGPT-STYLE PLAYGROUND</span><h2>Creative conversation</h2></div><select value={mode} onChange={e=>setMode(e.target.value)}>{[...TEXT_TYPES,["SEO","SEO Package"]].map(([k,l])=><option key={k} value={k}>{l}</option>)}</select></div>
            <div className="playground-chat-stream">
              {!chat.length?<div className="playground-empty"><Bot size={34}/><strong>Start with a business request</strong><span>“Create a festive offer for our customers with a warm WhatsApp message.”</span></div>:chat.map((m,i)=><div className={"playground-message "+m.role} key={i}><div className="playground-message-role">{m.role==="user"?"You":m.role==="assistant"?"GOLD-e AI":"System"}</div><div className="playground-message-body">{m.content}</div>{m.role==="assistant"&&<button className="mini-copy" onClick={()=>copyText(m.content)}>{copied?<Check size={12}/>:<Copy size={12}/>}Copy</button>}</div>)}
              {busy&&<div className="playground-message assistant"><div className="playground-message-role">GOLD-e AI</div><div className="thinking-dots">Thinking…</div></div>}
            </div>
            <div className="playground-composer"><textarea value={prompt} onChange={e=>setPrompt(e.target.value)} onKeyDown={e=>{if(e.key==="Enter"&&!e.shiftKey){e.preventDefault();runText();}}} placeholder="Tell GOLD-e what you want to create…"/><button className="command-btn compact" onClick={()=>runText()} disabled={busy||!prompt.trim()}><Send size={15}/>Create</button></div>
          </div>
          <aside className="playground-context panel"><div className="panel-kicker">BUSINESS CONTEXT</div><h3>Ground the output</h3><p>Give the AI facts it can safely use. It should not invent prices, availability or claims.</p><textarea value={context} onChange={e=>setContext(e.target.value)} placeholder="Business name, offer, service, location, price, audience, tone…"/><div className="context-chips"><button onClick={()=>setPrompt("Create a customer-friendly WhatsApp offer message with 3 quick replies.")}>Smart WhatsApp</button><button onClick={()=>setPrompt("Create an email promotion with subject, preheader and CTA.")}>Email</button><button onClick={()=>setPrompt("Create an SEO package for this business page.")}>SEO</button></div></aside>
        </section>}

        {tab==="images"&&<section className="playground-section">
          <div className="playground-creator panel"><div className="playground-section-head"><div><span className="panel-kicker">IMAGE CREATOR</span><h2>Banner, offer picture or logo</h2><p>Image generation only. GOLD-e Playground does not expose a video-generation option.</p></div></div>
            <div className="asset-type-grid">{IMAGE_TYPES.map(([k,l])=><button key={k} className={mode===k?"selected":""} onClick={()=>setMode(k)}><Image size={18}/><strong>{l}</strong><small>AI-generated image</small></button>)}</div>
            <textarea className="playground-large-input" value={prompt} onChange={e=>setPrompt(e.target.value)} placeholder="Describe the visual: brand colors, product, offer, layout, audience, text to appear…"/>
            <div className="playground-actions"><button className="command-btn" disabled={busy||!prompt.trim()} onClick={()=>generateImage(mode)}><Wand2 size={16}/>{busy?"Generating…":"Generate image"}</button></div>
          </div>
          <div className="playground-library panel"><div className="playground-section-head"><div><span className="panel-kicker">SHARE ON WHATSAPP</span><h2>Use a connected channel</h2></div><Share2 size={18}/></div>
            <div className="form-grid"><label className="inspector-label">WhatsApp channel<select value={selectedChannel} onChange={e=>setSelectedChannel(e.target.value)}><option value="">Select channel</option>{channels.map(c=><option key={c.id} value={c.id}>{c.display_name||c.external_account_id||c.id}</option>)}</select></label><label className="inspector-label">Contact group<select value={selectedGroup} onChange={e=>setSelectedGroup(e.target.value)}><option value="">Select imported group</option>{groups.map(g=><option key={g.id} value={g.id}>{g.name+" · "+g.contact_count}</option>)}</select></label></div>
            <label className="inspector-label">Caption<textarea value={caption} onChange={e=>setCaption(e.target.value)} placeholder="Optional WhatsApp caption"/></label>
            <div className="share-preview">{selectedAssetRow?.public_key?<img src={"https://api.goldetech.com/api/public/playground/assets/"+selectedAssetRow.public_key} alt={selectedAssetRow.title}/>:<span>Select a generated image from the library.</span>}</div>
            <button className="command-btn full-btn" disabled={busy||!selectedChannel||!selectedGroup||!selectedAssetRow?.public_key} onClick={share}><Share2 size={15}/>{busy?"Sending…":"Share to contact group"}</button>
          </div>
          <div className="playground-asset-grid">{imageAssets.map(a=><button key={a.id} className={selectedAsset===a.id?"selected":""} onClick={()=>setSelectedAsset(a.id)}><img src={"https://api.goldetech.com/api/public/playground/assets/"+a.public_key} alt={a.title}/><span>{a.kind.replaceAll("_"," ")}</span><strong>{a.title}</strong></button>)}</div>
        </section>}

        {tab==="templates"&&<section className="playground-section"><div className="template-workbench panel"><div className="playground-section-head"><div><span className="panel-kicker">MESSAGE FACTORY</span><h2>Email + WhatsApp templates</h2><p>Generate reusable copy. Smart WhatsApp outputs can include quick replies for bot and automation paths.</p></div></div><div className="asset-type-grid">{TEXT_TYPES.map(([k,l])=><button key={k} className={mode===k?"selected":""} onClick={()=>setMode(k)}><FileText size={18}/><strong>{l}</strong><small>Text generation</small></button>)}</div><textarea className="playground-large-input" value={prompt} onChange={e=>setPrompt(e.target.value)} placeholder="Describe the message, audience, offer, tone and desired action…"/><button className="command-btn" disabled={busy||!prompt.trim()} onClick={()=>runText(mode)}><Sparkles size={16}/>{busy?"Generating…":"Generate template"}</button>{chat.filter(m=>m.role==="assistant").slice(-1).map((m,i)=><div className="template-output" key={i}><div className="template-output-head"><strong>Generated {mode.replaceAll("_"," ")}</strong><button className="ghost-btn compact" onClick={()=>copyText(m.content)}><Copy size={13}/>Copy</button></div><pre>{m.content}</pre>{mode==="SMART_WHATSAPP"&&<SmartButtonEditor asset={m.asset} actions={smartActions} automations={automations} onSave={saveAction} />}</div>)}</div></section>}

        {tab==="seo"&&<section className="playground-section"><div className="seo-workbench panel"><div className="playground-section-head"><div><span className="panel-kicker">BUILT-IN SEO</span><h2>SEO Growth Assistant</h2><p>Generate a structured SEO package for pages, products and business content. SEO improves technical/content readiness; it does not guarantee rankings.</p></div><Search size={18}/></div><textarea className="playground-large-input" value={prompt} onChange={e=>setPrompt(e.target.value)} placeholder="Describe the page/business: service, city, audience, unique value, products…"/><button className="command-btn" disabled={busy||!prompt.trim()} onClick={()=>runText("SEO")}><Search size={15}/>{busy?"Generating…":"Generate SEO package"}</button>{chat.filter(m=>m.role==="assistant").slice(-1).map((m,i)=><div className="seo-output" key={i}><pre>{m.content}</pre></div>)}</div><div className="seo-checklist panel"><strong>SEO foundation included</strong><ul><li>Title + meta description</li><li>Keyword and intent mapping</li><li>H1/H2 content structure</li><li>FAQ opportunities</li><li>Open Graph copy</li><li>JSON-LD recommendation</li><li>Internal-link ideas</li></ul></div></section>}
      </main>
    </div>
  </div>;
}
export function ChannelContactImporter({ channel }) {
  const [file,setFile]=useState(null);
  const [groupName,setGroupName]=useState("");
  const [busy,setBusy]=useState(false);
  const [notice,setNotice]=useState("");
  const [error,setError]=useState("");

  if(!channel) return null;
  if(channel.channel!=="WHATSAPP") return <div className="inspector-note"><b>Contact import:</b> CSV-to-group import is currently enabled for WhatsApp channels. Instagram/Facebook contact imports can be added with their platform-specific permissions.</div>;

  const upload=async()=>{
    if(!file||busy)return;
    setBusy(true);setNotice("");setError("");
    try{
      const data=unwrap(await playgroundApi.importContacts(channel.id,file,groupName));
      setNotice("Imported "+(data?.imported||0)+" contacts into "+(data?.group_name||"the contact group")+".");
      setFile(null);setGroupName("");
    }catch(e){setError(e?.response?.data?.error||e?.message||"Contact import failed.");}
    finally{setBusy(false);}
  };

  return <div className="channel-import-box">
    <div><span className="panel-kicker">CONTACT GROUP IMPORT</span><strong>Import customers into a reusable group</strong><p>Upload a CSV with a <b>phone</b>, <b>mobile</b> or <b>whatsapp</b> column. Name/email/company are optional.</p></div>
    <div className="form-grid"><label className="inspector-label">Group name<input value={groupName} onChange={e=>setGroupName(e.target.value)} placeholder="Festival Offer Customers"/></label><label className="inspector-label">Contact file<input type="file" accept=".csv,text/csv" onChange={e=>setFile(e.target.files?.[0]||null)}/></label></div>
    {file&&<div className="import-file"><span>{file.name}</span><small>{Math.ceil(file.size/1024)} KB</small></div>}
    {error&&<div className="alert error">{error}</div>}
    {notice&&<div className="alert success">{notice}</div>}
    <button className="ghost-btn full-btn" disabled={!file||busy} onClick={upload}><Upload size={14}/>{busy?"Importing…":"Import contacts & create group"}</button>
  </div>;
}

export default Playground;
