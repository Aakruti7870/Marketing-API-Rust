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

function initials(user) {
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
      setError(err?.response?.data?.message || err?.response?.data?.error || err?.message || "Authentication failed.");
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
  return <div className="app-shell">
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

function Dashboard({ setPage }) {
  const [data,setData]=useState(null); const [loading,setLoading]=useState(true);
  useEffect(()=>{dashboardApi.get().then(r=>setData(unwrap(r))).catch(()=>{}).finally(()=>setLoading(false))},[]);
  const stats = data?.metrics || data?.summary || data || {};
  const cards = [
    [Zap,"Active campaigns",stats.active_campaigns ?? stats.activeCampaigns ?? 0,"Live","purple"],
    [Users,"Contacts",stats.contacts ?? stats.total_contacts ?? 0,"Workspace total","blue"],
    [MessageSquare,"Messages sent",stats.messages_sent ?? stats.sent_messages ?? 0,"Across channels","pink"],
    [Bot,"AI runs",stats.ai_runs ?? stats.agent_runs ?? 0,"Automation activity","gold"],
  ];
  return <div>
    <PageHeader eyebrow="COMMAND CENTER" title="Good morning. Your growth engine is ready." description="Monitor campaigns, AI execution, audience activity and messaging from one workspace." action={<button className="primary-btn compact" onClick={()=>setPage("agents")}><Sparkles size={16}/>Launch AI run</button>}/>
    {loading && <div className="loading-line"><span/>Syncing live workspace data…</div>}
    <div className="stat-grid">{cards.map(([I,l,v,c,t])=><StatCard key={l} icon={I} label={l} value={formatNumber(v)} change={c} tone={t}/>)}</div>
    <div className="dashboard-grid">
      <section className="panel hero-panel"><div className="panel-head"><div><span className="panel-kicker">AI OPERATIONS</span><h2>Marketing Command Center</h2></div><button className="ghost-btn" onClick={()=>setPage("analytics")}>View analytics <ChevronRight size={15}/></button></div><div className="signal"><div className="signal-ring"><div><Sparkles size={27}/><strong>AI</strong></div></div><div><h3>Orchestrate your next growth move</h3><p>Turn audience signals into campaign actions, approvals and measurable outcomes.</p><div className="signal-actions"><button onClick={()=>setPage("campaigns")}><Rocket size={16}/>Create campaign</button><button onClick={()=>setPage("agents")}><Bot size={16}/>Review AI runs</button></div></div></div></section>
      <section className="panel"><div className="panel-head"><div><span className="panel-kicker">ACTIVITY</span><h2>Workspace pulse</h2></div><Activity size={18} className="muted-icon"/></div><div className="activity-list"><div><i className="dot purple"/><span><strong>AI automation</strong><small>Ready for your next run</small></span><time>Now</time></div><div><i className="dot pink"/><span><strong>Campaign engine</strong><small>Campaign actions available</small></span><time>Live</time></div><div><i className="dot blue"/><span><strong>Audience layer</strong><small>Contacts connected</small></span><time>Live</time></div></div></section>
    </div>
    <section className="quick-grid"><button onClick={()=>setPage("agents")}><div><Bot/></div><span><strong>AI Agents</strong><small>Run, approve and monitor autonomous workflows.</small></span><ChevronRight/></button><button onClick={()=>setPage("campaigns")}><div><Target/></div><span><strong>Campaigns</strong><small>Launch and control multi-channel campaigns.</small></span><ChevronRight/></button><button onClick={()=>setPage("messages")}><div><MessageSquare/></div><span><strong>Messaging</strong><small>Dispatch and track customer messages.</small></span><ChevronRight/></button></section>
  </div>;
}

function SimplePage({ type }) {
  const [items,setItems]=useState([]); const [loading,setLoading]=useState(true);
  const configs={
    agents:{title:"AI Agents",eyebrow:"AUTOMATION",desc:"Run AI workflows and review human approval checkpoints.",api:agentsApi.runs,icon:Bot,empty:"No agent runs yet."},
    campaigns:{title:"Campaigns",eyebrow:"ORCHESTRATION",desc:"Create, launch and monitor your marketing campaigns.",api:campaignsApi.list,icon:Target,empty:"No campaigns yet."},
    contacts:{title:"Contacts",eyebrow:"AUDIENCE",desc:"Manage the workspace audience powering your growth engine.",api:contactsApi.list,icon:Users,empty:"No contacts yet."},
    messages:{title:"Messages",eyebrow:"CONVERSATIONS",desc:"Monitor dispatched messages and messaging activity.",api:messagesApi.list,icon:MessageSquare,empty:"No messages yet."},
  };
  const c=configs[type];
  useEffect(()=>{c.api({page:1,limit:20}).then(r=>{const x=r?.data;setItems(Array.isArray(x?.data)?x.data:Array.isArray(x)?x:[])}).catch(()=>setItems([])).finally(()=>setLoading(false))},[type]);
  return <div><PageHeader eyebrow={c.eyebrow} title={c.title} description={c.desc} action={<button className="primary-btn compact"><Plus size={16}/>Create new</button>}/><section className="panel table-panel">{loading?<div className="empty-state"><div className="spinner"/><h3>Loading workspace data</h3><p>Connecting to the Rust API…</p></div>:items.length===0?<div className="empty-state"><div className="empty-icon"><c.icon/></div><h3>{c.empty}</h3><p>This workspace is ready. Create your first {type==="agents"?"AI run":type.slice(0,-1)} to see activity here.</p><button className="primary-btn compact"><Plus size={16}/>Get started</button></div>:<div className="data-list">{items.map((item,i)=><div className="data-row" key={item.id||i}><div className="row-icon"><c.icon size={17}/></div><div><strong>{item.name||item.title||item.status||`Record ${i+1}`}</strong><small>{item.description||item.email||item.channel||item.created_at||"Workspace record"}</small></div><span className="status-pill">{item.status||"Active"}</span><ChevronRight size={16}/></div>)}</div>}</section></div>;
}

function AutomationsPage() {
  const [items,setItems]=useState([]),[loading,setLoading]=useState(true),[notice,setNotice]=useState("");
  const load=()=>{setLoading(true);automationsApi.list().then(unwrap).then(x=>setItems(Array.isArray(x?.data)?x.data:Array.isArray(x)?x:[])).catch(()=>setItems([])).finally(()=>setLoading(false))};
  useEffect(()=>{load()},[]);
  const act=async(id,action)=>{try{await automationsApi[action](id);setNotice(action==="publish"?"Automation published.":"Automation paused.");load();setTimeout(()=>setNotice(""),2500)}catch(e){setNotice(e?.response?.data?.error||e?.message||"Action failed.")}};
  return <div><PageHeader eyebrow="AUTOMATION ENGINE" title="Automations" description="Build native workflows natively inside GOLD-e GrowthOS." action={<button className="command-btn"><Plus size={16}/>Create automation</button>}/>{notice&&<div className="action-notice"><Check size={15}/>{notice}</div>}
    <section className="panel table-panel">{loading?<div className="empty-state"><div className="spinner"/><h3>Loading automations</h3></div>:items.length===0?<div className="empty-state"><div className="empty-icon"><Zap/></div><h3>No automations yet.</h3><p>Create your first workflow with triggers, HTTP actions, conditions and approval gates.</p><button className="command-btn"><Plus size={16}/>Build first automation</button></div>:<div className="data-list">{items.map(a=><div className="data-row" key={a.id}><div className="row-icon"><Zap size={17}/></div><div><strong>{a.name}</strong><small>{a.status} · v{a.version||1}</small></div><span className="status-pill">{a.status}</span>{a.status==="PUBLISHED"?<button className="row-action" onClick={()=>act(a.id,"pause")}><Pause size={13}/></button>:<button className="row-action" onClick={()=>act(a.id,"publish")}><Play size={13}/></button>}<button className="row-action" onClick={async()=>{await automationsApi.run(a.id,{payload:{source:"GrowthOS manual test"}});setNotice("Test execution completed.");setTimeout(()=>setNotice(""),2500)}}>Run</button></div>)}</div>}</section></div>;
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
  else if(page==="automations") content=<AutomationsPage/>;
  else if(page==="settings") content=<SettingsPage user={user}/>;
  else content=<SimplePage type={page}/>;

  return <Shell user={user} onLogout={logout} page={page} setPage={setPage}>{content}</Shell>;
}
