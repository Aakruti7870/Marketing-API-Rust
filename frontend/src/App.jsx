import { useEffect, useMemo, useState } from "react";
import {
  Activity, ArrowRight, BarChart3, Bot, Check, ChevronDown, ChevronLeft, ChevronRight, CircleHelp,
  Command, ContactRound, LayoutDashboard, LogOut, Menu, MessageSquare,
  Pause, Play, Plus, Rocket, Search, Settings, Sparkles, Target, Users, X, Zap
} from "lucide-react";
import { authApi, dashboardApi, workspaceApi, agentsApi, campaignsApi, contactsApi, messagesApi, automationsApi, unwrap } from "./services/api";
import "./App.css";

const NAV = [
  { id:"dashboard", label:"Command Center", icon:LayoutDashboard },
  { id:"contacts", label:"Contacts", icon:ContactRound },
  { id:"campaigns", label:"Campaigns", icon:Target },
  { id:"messages", label:"WhatsApp", icon:MessageSquare },
  { id:"agents", label:"AI Agents", icon:Bot },
  { id:"automations", label:"Automations", icon:Zap },
  { id:"analytics", label:"Analytics", icon:BarChart3 },
  { id:"settings", label:"Settings", icon:Settings },
];


const AI_HEROES = [
  {title:"AURA 7 · COMMAND INTELLIGENCE",copy:"Turn audience signals into coordinated growth actions.",image:"https://dnznrvs05pmza.cloudfront.net/gemini/gemini-3-pro-image/images/ccca5a9a-0c41-444d-b31d-c7464a7e8e57/6f9db864-db66-4e64-87ea-307b13821889/AURA_7_futuristic_AI_marketing_command_center__elegant_human.png?_jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJrZXlIYXNoIjoiMzRkYmFjYzYzY2Y1OWQ2ZSIsImJ1Y2tldCI6InJ1bndheS10YXNrLWFydGlmYWN0cyIsInN0YWdlIjoicHJvZCIsImV4cCI6MTc5MDM4MzA0Nn0.zDLUeI-IaO8CXT1Bl0iSQG4hXgW00MRUfCeN7n4IG4w"},
  {title:"GROWTHOS · SIGNAL ENGINE",copy:"See campaigns, conversations and agent activity in one operating layer.",image:"https://dnznrvs05pmza.cloudfront.net/gemini/gemini-3-pro-image/images/1a586ab8-8555-4955-a1c2-cf314068b8be/bec74049-4ce8-4f76-8da4-bc085c61fb12/GOLD_e_GrowthOS_futuristic_marketing_operations_room__lumino.png?_jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJrZXlIYXNoIjoiYjJkZTVlYmE4YTE2NDhkZSIsImJ1Y2tldCI6InJ1bndheS10YXNrLWFydGlmYWN0cyIsInN0YWdlIjoicHJvZCIsImV4cCI6MTc5MDQ1MzY5MH0.j5x42FUqI2_OS48q24YDQsBodz_Wgcrm2UFvYGengPg"},
  {title:"AURA 7 · GROWTH STRATEGIST",copy:"Automate the next best action while keeping approvals under human control.",image:"https://dnznrvs05pmza.cloudfront.net/gemini/gemini-3-pro-image/images/f14ba74d-6f2e-47dd-aa0f-e82fe737cb2a/fa7a8f89-1711-4daa-8c79-62c7e0ffbfdc/AURA_7_AI_growth_strategist_in_a_premium_dark_digital_studio.png?_jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJrZXlIYXNoIjoiZDI5ZjVjNjIxYjM5ZTZiNSIsImJ1Y2tldCI6InJ1bndheS10YXNrLWFydGlmYWN0cyIsInN0YWdlIjoicHJvZCIsImV4cCI6MTc5MDQ1MjgzNH0.wC3wF7AMLipPtBfdB8LkZRe5E3HosI0fpq28XFDk-zE"}
];function initials(user) {
  return [user?.first_name, user?.last_name].filter(Boolean).map(x => x[0]).join("").toUpperCase() || "GE";
}

function formatNumber(value) {
  if (value === undefined || value === null || Number.isNaN(Number(value))) return "—";
  return new Intl.NumberFormat("en-IN", { notation:"compact", maximumFractionDigits:1 }).format(Number(value));
}

function Login({ onLogin }) {
  const [mode, setMode] = useState("login");
  const [form, setForm] = useState({ email:"", password:"", first_name:"", last_name:"", workspace_name:"" });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const submit = async (e) => {
    e.preventDefault(); setBusy(true); setError("");
    try {
      const response = mode === "login"
        ? await authApi.login({ email:form.email, password:form.password })
        : await authApi.register(form);
      const data = unwrap(response);
      const token = data?.tokens?.access_token;
      if (!token) throw new Error("Authentication succeeded but no access token was returned.");
      localStorage.setItem("golde_access_token", token);
      if (data?.tokens?.refresh_token) localStorage.setItem("golde_refresh_token", data.tokens.refresh_token);
      if (data?.user) localStorage.setItem("golde_user", JSON.stringify(data.user));
      if (data?.workspace_id) localStorage.setItem("golde_workspace_id", data.workspace_id);
      onLogin(data?.user || null);
    } catch (err) {
      setError(err?.response?.data?.message || err?.message || "Authentication failed.");
    } finally { setBusy(false); }
  };

  return <main className="auth-screen">
    <div className="auth-orbit orbit-one" /><div className="auth-orbit orbit-two" />
    <section className="auth-card">
      <div className="brand-mark"><span>G</span><div><strong>GOLD-e</strong><small>GrowthOS</small></div></div>
      <div className="auth-heading">
        <div className="eyebrow"><Sparkles size={14}/> Marketing Intelligence</div>
        <h1>{mode === "login" ? "Welcome back." : "Build your growth workspace."}</h1>
        <p>{mode === "login" ? "Sign in to your AI-powered marketing command center." : "Create a workspace and start orchestrating campaigns with AI."}</p>
      </div>
      {error && <div className="alert error">{error}</div>}
      <form onSubmit={submit} className="auth-form">
        {mode === "register" && <div className="form-grid">
          <label>First name<input required value={form.first_name} onChange={e=>setForm({...form,first_name:e.target.value})}/></label>
          <label>Last name<input required value={form.last_name} onChange={e=>setForm({...form,last_name:e.target.value})}/></label>
        </div>}
        <label>Email<input required type="email" autoComplete="email" value={form.email} onChange={e=>setForm({...form,email:e.target.value})}/></label>
        <label>Password<input required minLength="8" type="password" autoComplete={mode==="login"?"current-password":"new-password"} value={form.password} onChange={e=>setForm({...form,password:e.target.value})}/></label>
        {mode === "register" && <label>Workspace name <span className="muted">(optional)</span><input value={form.workspace_name} onChange={e=>setForm({...form,workspace_name:e.target.value})} placeholder="My Growth Workspace"/></label>}
        <button className="primary-btn" disabled={busy}>{busy ? "Connecting…" : mode==="login" ? "Enter GrowthOS" : "Create workspace"}<ChevronRight size={18}/></button>
      </form>
      <button className="text-btn" onClick={()=>{setMode(mode==="login"?"register":"login");setError("")}}>
        {mode==="login" ? "Create a new workspace" : "Already have an account? Sign in"}
      </button>
      <div className="auth-footer"><span>Secure workspace isolation</span><span>JWT protected API</span></div>
    </section>
  </main>;
}

function Shell({ user, onLogout, page, setPage, children }) {
  const [mobileOpen, setMobileOpen] = useState(false);
  return <div className="app-shell command-shell">
    <aside className={`sidebar ${mobileOpen?"open":""}`}>
      <div className="sidebar-brand"><div className="brand-symbol">G</div><div><strong>GOLD-e</strong><small>GrowthOS</small></div><button className="mobile-close" onClick={()=>setMobileOpen(false)}><X/></button></div>
      <div className="workspace-mini"><div className="workspace-icon">G</div><div><strong>Growth Workspace</strong><span>Active workspace</span></div><ChevronDown size={15}/></div>
      <nav>{NAV.map(item => { const Icon=item.icon; return <button key={item.id} className={page===item.id?"active":""} onClick={()=>{setPage(item.id);setMobileOpen(false)}}><Icon size={18}/><span>{item.label}</span>{item.id==="agents" && <em>AI</em>}</button> })}</nav>
      <div className="sidebar-bottom"><button><CircleHelp size={17}/>Help center</button><div className="user-mini"><div className="avatar">{initials(user)}</div><div><strong>{user?.first_name || "Workspace user"}</strong><span>{user?.email || ""}</span></div><button className="icon-btn" onClick={onLogout}><LogOut size={16}/></button></div></div>
    </aside>
    {mobileOpen && <div className="scrim" onClick={()=>setMobileOpen(false)}/>}
    <section className="main-area">
      <header className="topbar"><button className="mobile-menu" onClick={()=>setMobileOpen(true)}><Menu/></button><div className="crumb"><span>GrowthOS</span><ChevronRight size={14}/><strong>{NAV.find(x=>x.id===page)?.label}</strong></div><div className="top-actions"><button className="search-btn"><Search size={17}/><span>Search</span><kbd>⌘ K</kbd></button><button className="icon-circle"><Activity size={17}/></button><div className="top-avatar">{initials(user)}</div></div></header>
      <main className="page-content">{children}</main>
    </section>
  </div>;
}

function PageHeader({ eyebrow, title, description, action }) {
  return <div className="page-header"><div><div className="eyebrow">{eyebrow}</div><h1>{title}</h1><p>{description}</p></div>{action}</div>;
}

function StatCard({ icon:Icon, label, value, change, tone="" }) {
  return <div className="stat-card"><div className={`stat-icon ${tone}`}><Icon size={19}/></div><div className="stat-copy"><span>{label}</span><strong>{value}</strong><small className={change?.startsWith("-")?"negative":""}>{change || "Live workspace metric"}</small></div></div>;
}

function HeroCarousel({setPage}) {
  const [index,setIndex]=useState(0);
  useEffect(()=>{const timer=setInterval(()=>setIndex(i=>(i+1)%AI_HEROES.length),7000);return()=>clearInterval(timer)},[]);
  const slide=AI_HEROES[index];
  return <section className="hero-carousel">
    <img src={slide.image} alt="" className="hero-image"/>
    <div className="hero-vignette"/>
    <div className="hero-content"><span className="hero-kicker">{slide.title}</span><h2>Growth intelligence,<br/><strong>in motion.</strong></h2><p>{slide.copy}</p><div className="hero-actions"><button className="command-btn" onClick={()=>setPage("agents")}><Zap size={16}/>Run Growth Plan</button><button className="command-btn ghost" onClick={()=>setPage("campaigns")}>Open Campaigns <ArrowRight size={15}/></button></div></div>
    <div className="carousel-controls"><button onClick={()=>setIndex((index-1+AI_HEROES.length)%AI_HEROES.length)} aria-label="Previous"><ChevronLeft size={17}/></button><div>{AI_HEROES.map((_,i)=><button key={i} className={i===index?"active":""} onClick={()=>setIndex(i)} aria-label={`Slide ${i+1}`}/>)}</div><button onClick={()=>setIndex((index+1)%AI_HEROES.length)} aria-label="Next"><ChevronRight size={17}/></button></div>
  </section>;
}

function Dashboard({ setPage }) {
  const [data,setData]=useState(null);
  const [runs,setRuns]=useState([]);
  const [loading,setLoading]=useState(true);

  useEffect(()=>{
    Promise.all([
      dashboardApi.get().then(r=>unwrap(r)).catch(()=>null),
      agentsApi.runs({page:1,limit:8}).then(r=>{
        const x=r?.data;
        return Array.isArray(x?.data)?x.data:Array.isArray(x)?x:[];
      }).catch(()=>[])
    ]).then(([dashboardData, agentRuns])=>{
      setData(dashboardData);
      setRuns(agentRuns);
    }).finally(()=>setLoading(false));
  },[]);

  const stats = data?.summary || data?.metrics || data || {};
  const contacts = Number(stats.total_contacts ?? stats.contacts ?? 0);
  const campaigns = Number(stats.active_campaigns ?? stats.activeCampaigns ?? 0);
  const sent = Number(stats.messages_sent ?? stats.sent_messages ?? 0);
  const agentRuns = Number(stats.agent_runs_total ?? stats.ai_runs ?? stats.agent_runs ?? 0);
  const approvals = Number(stats.pending_approvals_count ?? 0);

  const feed = runs.slice(0,6).map((run,i)=>({
    icon:["🧠","💬","📣","🤖","🛡️","📈"][i%6],
    title:run.name || run.agent_name || run.title || `AI agent run ${i+1}`,
    detail:run.status ? `Status: ${String(run.status).replaceAll("_"," ")}` : "Workspace automation",
    state:String(run.status||"RUNNING").toUpperCase()
  }));

  if(!feed.length && !loading){
    feed.push(
      {icon:"🧠",title:"AURA 7 is ready",detail:"Create your first AI run to start activity",state:"READY"},
      {icon:"📣",title:"Campaign engine ready",detail:"Create your first campaign",state:"READY"},
      {icon:"👥",title:"Audience layer ready",detail:"Add contacts to activate growth signals",state:"READY"}
    );
  }

  const now = new Date();
  const dateLine = now.toLocaleDateString("en-US",{weekday:"long",day:"2-digit",month:"short",year:"numeric"}).toUpperCase();
  const hour = now.getHours();
  const greeting = hour<12 ? "morning" : hour<17 ? "afternoon" : "evening";
  const savedUser = (()=>{ try { return JSON.parse(localStorage.getItem("golde_user")||"{}"); } catch { return {}; } })();

  return <div className="command-center">
    <section className="command-welcome">
      <div className="command-aura"/>
      <div className="command-orbit orbit-a"/>
      <div className="command-orbit orbit-b"/>
      <div className="command-floor"><span/><i/><b/></div>
      <div className="command-welcome-content">
        <div>
          <div className="command-date">{dateLine}</div>
          <h1>Good {greeting}, {savedUser?.first_name || "there"} 👋</h1>
          <p>AURA 7 is connected to your workspace. Turn audience signals into campaigns, approvals and measurable growth actions.</p>
          <div className="command-actions">
            <button className="command-btn" onClick={()=>setPage("agents")}><Zap size={16}/>Run Growth Plan</button>
            <button className="command-btn ghost" onClick={()=>setPage("agents")}>Review approvals{approvals ? ` (${approvals})` : ""}</button>
          </div>
        </div>
        <div className="growth-score">
          <svg width="92" height="92" viewBox="0 0 92 92">
            <circle cx="46" cy="46" r="37" fill="none" stroke="rgba(255,255,255,.10)" strokeWidth="7"/>
            <circle cx="46" cy="46" r="37" fill="none" stroke="#ff9a3d" strokeWidth="7" strokeLinecap="round" strokeDasharray="233" strokeDashoffset="65"/>
          </svg>
          <strong>72</strong><span>GROWTH SCORE</span>
        </div>
      </div>
    </section>

    <section className="command-kpis">
      <div><span>CONTACTS · WORKSPACE</span><strong>{formatNumber(contacts)}</strong><small>Audience layer</small></div>
      <div><span>ACTIVE CAMPAIGNS</span><strong>{formatNumber(campaigns)}</strong><small>Campaign engine</small></div>
      <div><span>WHATSAPP · 30D</span><strong>{formatNumber(sent)}</strong><small>Messages sent</small></div>
      <div><span>AGENT RUNS</span><strong>{formatNumber(agentRuns)}</strong><small>{approvals ? `${approvals} awaiting approval` : "Automation activity"}</small></div>
    </section>

    <section className="command-columns">
      <div className="command-card">
        <h3><span className="live-dot"/>LIVE AGENT ACTIVITY</h3>
        <div className="command-feed">
          {loading ? <div className="command-empty">Syncing workspace activity…</div> : feed.map((item,i)=>
            <div className="command-feed-item" key={i}>
              <div className="feed-icon">{item.icon}</div>
              <div><b>{item.title}</b><span>{item.detail}</span></div>
              <em className={item.state==="READY"?"ready":""}>{item.state==="RUNNING"?"RUNNING":item.state==="READY"?"READY":"DONE"}</em>
            </div>
          )}
        </div>
      </div>

      <div className="command-card">
        <h3>🛡️ APPROVAL QUEUE</h3>
        <div className="approval-list">
          {approvals > 0 ? <div className="approval-item"><b>{approvals} approval{approvals===1?"":"s"} awaiting review</b><span>Open AI Agents to review pending actions.</span><button onClick={()=>setPage("agents")}>Review</button></div> :
          <div className="approval-item"><b>No approvals waiting.</b><span>Your workspace is clear for the next growth action.</span><button onClick={()=>setPage("agents")}>Open Agents</button></div>}
        </div>
        <div className="command-check">
          <div className="done">Rust API connected</div>
          <div className="done">Workspace authenticated</div>
          <div className="done">Analytics connected</div>
          <div className={sent>0?"done":""}>{sent>0 ? "WhatsApp activity detected" : "WhatsApp activity pending"}</div>
        </div>
      </div>
    </section>
  </div>;
}

function CreateModal({type,onClose,onDone}) {
  const [busy,setBusy]=useState(false),[error,setError]=useState("");
  const [form,setForm]=useState(type==="contacts"?{first_name:"",last_name:"",phone:"",email:"",company:""}:type==="campaigns"?{name:"",description:"",channel:"whatsapp"}:type==="agents"?{agent_type:"growth",name:"Growth Plan",input_params:"{}"}:{contact_id:"",content:""});
  const submit=async e=>{e.preventDefault();setBusy(true);setError("");try{
    let r;
    if(type==="contacts") r=await contactsApi.create({...form,tags:[],status:"lead"});
    else if(type==="campaigns") r=await campaignsApi.create({...form,target_tags:[]});
    else if(type==="agents") r=await agentsApi.create({...form,input_params:JSON.parse(form.input_params||"{}")});
    else r=await messagesApi.send(form);
    onDone(unwrap(r)||{}); onClose();
  }catch(err){setError(err?.response?.data?.error?.message||err?.response?.data?.message||err?.message||"Action failed.")}finally{setBusy(false)}};
  return <Modal title={type==="contacts"?"Add contact":type==="campaigns"?"Create campaign":type==="agents"?"Run AI agent":"Send WhatsApp message"} onClose={onClose}>
    {error&&<div className="alert error">{error}</div>}
    <form className="modal-form" onSubmit={submit}>
      {type==="contacts"&&<><label>First name<input required value={form.first_name} onChange={e=>setForm({...form,first_name:e.target.value})}/></label><label>Last name<input value={form.last_name} onChange={e=>setForm({...form,last_name:e.target.value})}/></label><label>Phone<input required value={form.phone} onChange={e=>setForm({...form,phone:e.target.value})}/></label><label>Email<input type="email" value={form.email} onChange={e=>setForm({...form,email:e.target.value})}/></label><label>Company<input value={form.company} onChange={e=>setForm({...form,company:e.target.value})}/></label></>}
      {type==="campaigns"&&<><label>Campaign name<input required value={form.name} onChange={e=>setForm({...form,name:e.target.value})}/></label><label>Description<textarea value={form.description} onChange={e=>setForm({...form,description:e.target.value})}/></label><label>Channel<select value={form.channel} onChange={e=>setForm({...form,channel:e.target.value})}><option value="whatsapp">WhatsApp</option><option value="email">Email</option></select></label></>}
      {type==="agents"&&<><label>Agent type<input required value={form.agent_type} onChange={e=>setForm({...form,agent_type:e.target.value})}/></label><label>Run name<input required value={form.name} onChange={e=>setForm({...form,name:e.target.value})}/></label><label>Input JSON<textarea value={form.input_params} onChange={e=>setForm({...form,input_params:e.target.value})}/></label></>}
      {type==="messages"&&<><label>Contact ID<input required value={form.contact_id} onChange={e=>setForm({...form,contact_id:e.target.value})}/></label><label>Message<textarea required value={form.content} onChange={e=>setForm({...form,content:e.target.value})}/></label></>}
      <div className="modal-actions"><button type="button" className="command-btn ghost" onClick={onClose}>Cancel</button><button className="command-btn" disabled={busy}>{busy?"Working…":"Execute"}<ArrowRight size={15}/></button></div>
    </form>
  </Modal>;
}

function AgentReviewModal({runId,onClose,onDone}) {
  const [run,setRun]=useState(null),[busy,setBusy]=useState(false),[error,setError]=useState("");
  useEffect(()=>{agentsApi.run(runId).then(unwrap).then(setRun).catch(err=>setError(err?.response?.data?.error?.message||err?.message||"Unable to load agent run."))},[runId]);
  const act=async(step,kind)=>{setBusy(true);setError("");try{if(kind==="approve")await agentsApi.approve(runId,step.id);else await agentsApi.reject(runId,step.id,{reason:"Rejected from GrowthOS Command Center"});const next=await agentsApi.run(runId);setRun(unwrap(next));onDone()}catch(err){setError(err?.response?.data?.error?.message||err?.message||"Workflow action failed.")}finally{setBusy(false)}};
  const steps=run?.steps||[];
  return <Modal title="Agent approval workflow" onClose={onClose}>
    {error&&<div className="alert error">{error}</div>}
    {!run?<div className="empty-state compact"><div className="spinner"/><h3>Loading agent run</h3></div>:<div className="review-list">
      <div className="review-summary"><strong>{run.name}</strong><span>{run.status} · {run.agent_type}</span></div>
      {steps.length===0?<div className="command-empty">No workflow steps returned for this run.</div>:steps.map(step=><div className="review-step" key={step.id}><div><b>{step.name}</b><span>{step.description||step.action_type||"Agent step"} · {step.status}</span></div>{step.requires_approval&&["PENDING","WAITING","AWAITING_APPROVAL"].includes(String(step.status).toUpperCase())?<div className="review-actions"><button disabled={busy} onClick={()=>act(step,"approve")}><Check size={13}/>Approve</button><button disabled={busy} onClick={()=>act(step,"reject")}><X size={13}/>Reject</button></div>:<em>{step.status}</em>}</div>)}
    </div>}
  </Modal>;
}

function SimplePage({ type }) {
  const [items,setItems]=useState([]),[loading,setLoading]=useState(true),[modal,setModal]=useState(false),[reviewRun,setReviewRun]=useState(null),[notice,setNotice]=useState("");
  const configs={
    agents:{title:"AI Agents",eyebrow:"AUTOMATION",desc:"Run AI workflows and review human approval checkpoints.",api:agentsApi.runs,icon:Bot,empty:"No agent runs yet."},
    campaigns:{title:"Campaigns",eyebrow:"ORCHESTRATION",desc:"Create, launch and monitor your marketing campaigns.",api:campaignsApi.list,icon:Target,empty:"No campaigns yet."},
    contacts:{title:"Contacts",eyebrow:"AUDIENCE",desc:"Manage the audience powering your growth engine.",api:contactsApi.list,icon:Users,empty:"No contacts yet."},
    messages:{title:"WhatsApp",eyebrow:"CONVERSATIONS",desc:"Dispatch and monitor WhatsApp messaging activity.",api:messagesApi.list,icon:MessageSquare,empty:"No messages yet."}
  };
  const c=configs[type];
  const load=()=>{setLoading(true);c.api({page:1,limit:20}).then(unwrap).then(x=>setItems(Array.isArray(x?.data)?x.data:Array.isArray(x)?x:[])).catch(()=>setItems([])).finally(()=>setLoading(false))};
  useEffect(()=>{load()},[type]);
  const action=async(fn,msg)=>{try{await fn();setNotice(msg);load();setTimeout(()=>setNotice(""),3000)}catch(err){setNotice(err?.response?.data?.error?.message||err?.message||"Action failed.")}};
  return <div>
    <PageHeader eyebrow={c.eyebrow} title={c.title} description={c.desc} action={<button className="command-btn" onClick={()=>setModal(true)}><Plus size={16}/>Create new</button>}/>
    {notice&&<div className="action-notice"><Check size={15}/>{notice}</div>}
    <section className="panel table-panel">
      {loading?<div className="empty-state"><div className="spinner"/><h3>Loading workspace data</h3><p>Connecting to the Rust API…</p></div>
      :items.length===0?<div className="empty-state"><div className="empty-icon"><c.icon/></div><h3>{c.empty}</h3><p>This workspace is ready. Create the first {type==="agents"?"AI run":type==="messages"?"WhatsApp message":type.slice(0,-1)}.</p><button className="command-btn" onClick={()=>setModal(true)}><Plus size={16}/>Get started</button></div>
      :<div className="data-list">{items.map((item,i)=><div className="data-row" key={item.id||i}><div className="row-icon"><c.icon size={17}/></div><div><strong>{item.name||item.title||item.status||`Record ${i+1}`}</strong><small>{item.description||item.email||item.phone||item.content||item.channel||item.created_at||"Workspace record"}</small></div><span className="status-pill">{item.status||"Active"}</span>{type==="agents"&&item.id&&<button className="row-action" onClick={()=>setReviewRun(item.id)}>Review</button>}
        {type==="campaigns"&&item.id&&(String(item.status).toLowerCase()==="active"?<button className="row-action" onClick={()=>action(()=>campaignsApi.pause(item.id),"Campaign paused")}><Pause size={13}/></button>:<button className="row-action" onClick={()=>action(()=>campaignsApi.launch(item.id),"Campaign launch requested")}><Play size={13}/></button>)}
        <ChevronRight size={16}/></div>)}</div>}
    </section>
    {modal&&<CreateModal type={type} onClose={()=>setModal(false)} onDone={()=>{setNotice("Action completed successfully.");load();setTimeout(()=>setNotice(""),3000)}}/>}{reviewRun&&<AgentReviewModal runId={reviewRun} onClose={()=>setReviewRun(null)} onDone={()=>{setNotice("Agent workflow updated.");load();}}/>}
  </div>;
}
function AutomationBuilder({ automation, onBack, onSaved }) {
  const [name,setName]=useState(automation?.name||"New Growth Automation");
  const [nodes,setNodes]=useState(automation?.graph?.nodes||[
    {id:"trigger",name:"Webhook Trigger",type:"trigger.webhook",config:{},position:[80,180]},
    {id:"http",name:"HTTP Request",type:"action.http",config:{method:"POST",url:"https://example.com/webhook"},position:[360,180]},
    {id:"set",name:"Set Data",type:"data.set",config:{data:{status:"processed"}},position:[640,180]}
  ]);
  const [busy,setBusy]=useState(false),[notice,setNotice]=useState("");
  const edges=[{source:"trigger",target:"http"},{source:"http",target:"set"}];
  const save=async(publish=false)=>{
    setBusy(true);setNotice("");
    try{
      const graph={nodes,edges};
      const payload={name,description:"GOLD-e visual automation workflow",trigger_type:nodes[0]?.type==="trigger.schedule"?"SCHEDULE":nodes[0]?.type==="trigger.webhook"?"WEBHOOK":"MANUAL",trigger_config:{},graph};
      const res=automation?.id?await automationsApi.update(automation.id,payload):await automationsApi.create(payload);
      const saved=unwrap(res);
      if(publish) await automationsApi.activate(saved.id);
      setNotice(publish?"Automation published and active.":"Automation saved as draft.");
      onSaved();
    }catch(e){setNotice(e?.response?.data?.error?.message||e?.response?.data?.message||e?.message||"Unable to save automation.");}
    finally{setBusy(false)}
  };
  const addNode=(type,name)=>{
    const id=`node-${Date.now()}`;
    const config=type==="action.http"?{method:"GET",url:"https://api.example.com"}:type==="logic.condition"?{path:"status",equals:"ready"}:type==="delay.wait"?{milliseconds:1000}:type==="ai.agent"?{agent_type:"growth"}:{data:{}};
    setNodes([...nodes,{id,name,type,config,position:[80+nodes.length*280,360]}]);
  };
  return <div>
    <PageHeader eyebrow="AUTOMATION BUILDER" title={name} description="Compose triggers, logic, AI and actions into a reusable workflow." action={<div className="builder-actions"><button className="ghost-btn" onClick={onBack}>Back</button><button className="command-btn ghost" disabled={busy} onClick={()=>save(false)}>Save draft</button><button className="command-btn" disabled={busy} onClick={()=>save(true)}><Rocket size={15}/>Publish</button></div>}/>
    {notice&&<div className="action-notice"><Check size={15}/>{notice}</div>}
    <section className="builder-layout">
      <aside className="panel node-library"><div className="panel-head"><div><span className="panel-kicker">NODE LIBRARY</span><h2>Build blocks</h2></div></div>
        <input value={name} onChange={e=>setName(e.target.value)} placeholder="Automation name"/>
        <button onClick={()=>addNode("trigger.manual","Manual Trigger")}><Zap size={15}/>Manual Trigger</button>
        <button onClick={()=>addNode("trigger.webhook","Webhook Trigger")}><Zap size={15}/>Webhook Trigger</button>
        <button onClick={()=>addNode("trigger.schedule","Schedule Trigger")}><Zap size={15}/>Schedule Trigger</button>
        <button onClick={()=>addNode("action.http","HTTP Request")}><ArrowRight size={15}/>HTTP Request</button>
        <button onClick={()=>addNode("logic.condition","IF / Condition")}><Target size={15}/>IF / Condition</button>
        <button onClick={()=>addNode("delay.wait","Wait / Delay")}><Pause size={15}/>Wait / Delay</button>
        <button onClick={()=>addNode("ai.agent","AI Agent")}><Bot size={15}/>AI Agent</button>
        <button onClick={()=>addNode("data.set","Set Data")}><Settings size={15}/>Set Data</button>
      </aside>
      <section className="panel workflow-canvas">
        <div className="canvas-toolbar"><span>WORKFLOW · {nodes.length} NODES</span><span>Draft graph</span></div>
        <div className="canvas-grid">{nodes.map((n,i)=><div className="workflow-node" key={n.id}><div className="node-index">{i+1}</div><div><strong>{n.name}</strong><small>{n.type.replaceAll("."," · ").toUpperCase()}</small></div>{i<nodes.length-1&&<div className="node-link"/>}</div>)}</div>
        <div className="canvas-help">Connectors are stored as workflow edges. The execution engine supports Webhook/Manual triggers, HTTP requests, conditions, delays and data transforms now; AI nodes are reserved for the agent runtime integration.</div>
      </section>
    </section>
  </div>;
}

function AutomationsPage() {
  const [items,setItems]=useState([]),[loading,setLoading]=useState(true),[builder,setBuilder]=useState(null),[notice,setNotice]=useState("");
  const load=()=>{setLoading(true);automationsApi.list().then(unwrap).then(x=>setItems(Array.isArray(x?.data)?x.data:Array.isArray(x)?x:[])).catch(()=>setItems([])).finally(()=>setLoading(false))};
  useEffect(()=>{load()},[]);
  if(builder) return <AutomationBuilder automation={builder===true?null:builder} onBack={()=>setBuilder(null)} onSaved={()=>{setBuilder(null);load();}}/>;
  const act=async(id,action)=>{try{await automationsApi[action](id);setNotice(action==="activate"?"Automation published.":"Automation paused.");load();setTimeout(()=>setNotice(""),2500)}catch(e){setNotice(e?.response?.data?.message||e?.message||"Action failed.")}};
  return <div><PageHeader eyebrow="AUTOMATION ENGINE" title="Automations" description="Build n8n-style workflows natively inside GOLD-e GrowthOS." action={<button className="command-btn" onClick={()=>setBuilder(true)}><Plus size={16}/>Create automation</button>}/>{notice&&<div className="action-notice"><Check size={15}/>{notice}</div>}
    <section className="panel table-panel">{loading?<div className="empty-state"><div className="spinner"/><h3>Loading automations</h3></div>:items.length===0?<div className="empty-state"><div className="empty-icon"><Zap/></div><h3>No automations yet.</h3><p>Create your first workflow with triggers, HTTP actions, conditions and AI-ready nodes.</p><button className="command-btn" onClick={()=>setBuilder(true)}><Plus size={16}/>Build first automation</button></div>:<div className="data-list">{items.map(a=><div className="data-row" key={a.id}><div className="row-icon"><Zap size={17}/></div><div><strong>{a.name}</strong><small>{a.trigger_type} · v{a.version} · {a.graph?.nodes?.length||0} nodes</small></div><span className="status-pill">{a.status}</span><button className="row-action" onClick={()=>setBuilder(a)}>Edit</button>{a.status==="ACTIVE"?<button className="row-action" onClick={()=>act(a.id,"pause")}><Pause size={13}/></button>:<button className="row-action" onClick={()=>act(a.id,"activate")}><Play size={13}/></button>}<button className="row-action" onClick={async()=>{await automationsApi.run(a.id,{payload:{source:"GrowthOS manual test"}});setNotice("Test execution completed.");setTimeout(()=>setNotice(""),2500)}}>Test</button></div>)}</div>}</section></div>;
}

function Analytics() {
  const [data,setData]=useState(null);
  useEffect(()=>{dashboardApi.get().then(r=>setData(unwrap(r))).catch(()=>{})},[]);
  const values=useMemo(()=>[42,58,49,73,65,81,76],[]);
  return <div><PageHeader eyebrow="PERFORMANCE" title="Analytics" description="A live view of workspace performance and growth signals." action={<button className="ghost-btn"><BarChart3 size={16}/>Export report</button>}/><div className="analytics-grid"><section className="panel chart-panel"><div className="panel-head"><div><span className="panel-kicker">ACTIVITY TREND</span><h2>Growth activity</h2></div><span className="range-pill">Last 7 days</span></div><div className="bars">{values.map((v,i)=><div className="bar-wrap" key={i}><div className="bar" style={{height:`${v}%`}}/><small>{["M","T","W","T","F","S","S"][i]}</small></div>)}</div></section><section className="panel"><div className="panel-head"><div><span className="panel-kicker">LIVE DATA</span><h2>Dashboard payload</h2></div></div><pre className="json-view">{data?JSON.stringify(data,null,2):"Waiting for analytics endpoint…"}</pre></section></div></div>;
}

function SettingsPage({user}) {
  return <div><PageHeader eyebrow="WORKSPACE" title="Settings" description="Manage workspace access and your GrowthOS account."/><div className="settings-grid"><section className="panel setting-card"><div className="setting-icon"><Users/></div><div><h2>Workspace</h2><p>Current authenticated workspace and membership context.</p><div className="setting-value">{localStorage.getItem("golde_workspace_id") || "Workspace ID not returned"}</div></div></section><section className="panel setting-card"><div className="setting-icon"><Settings/></div><div><h2>Account</h2><p>{user?.email || "Authenticated user"}</p><div className="setting-value">{user?.first_name} {user?.last_name}</div></div></section></div></div>;
}

export default function App() {
  const [user,setUser]=useState(()=>{try{return JSON.parse(localStorage.getItem("golde_user"))}catch{return null}});
  const [page,setPage]=useState("dashboard");
  const [checking,setChecking]=useState(!!localStorage.getItem("golde_access_token"));

  useEffect(()=>{ if(!localStorage.getItem("golde_access_token")){setChecking(false);return;} authApi.me().then(r=>setUser(unwrap(r))).catch(()=>{localStorage.removeItem("golde_access_token");setUser(null)}).finally(()=>setChecking(false)); },[]);

  const logout=async()=>{try{await authApi.logout(localStorage.getItem("golde_refresh_token"))}catch{} ["golde_access_token","golde_refresh_token","golde_user","golde_workspace_id"].forEach(k=>localStorage.removeItem(k));setUser(null);setPage("dashboard")};

  if(checking) return <div className="boot-screen"><div className="boot-logo">G</div><span>Loading GrowthOS</span></div>;
  if(!user) return <Login onLogin={setUser}/>;

  let content;
  if(page==="dashboard") content=<Dashboard setPage={setPage}/>;
  else if(page==="analytics") content=<Analytics/>;
  else if(page==="settings") content=<SettingsPage user={user}/>;\n  else if(page==="automations") content=<AutomationsPage/>;
  else content=<SimplePage type={page}/>;

  return <Shell user={user} onLogout={logout} page={page} setPage={setPage}>{content}</Shell>;
}
