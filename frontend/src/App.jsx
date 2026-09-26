import { useEffect, useMemo, useState } from "react";
import {
  Activity, ArrowRight, BarChart3, Bot, Check, ChevronDown, ChevronLeft, ChevronRight, CircleHelp,
  Command, ContactRound, LayoutDashboard, LogOut, Menu, MessageSquare,
  Pause, Play, Plus, Rocket, Search, Settings, Sparkles, Target, Users, X, Zap,
  Clock3, Globe2, GitBranch, GripVertical, Save, ZoomIn, ZoomOut, Maximize2, Webhook, MousePointer2, PanelRight
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


const AUTOMATION_NODE_TYPES = [
  { type:"MANUAL_TRIGGER", label:"Manual Trigger", group:"Triggers", icon:MousePointer2, tone:"purple", description:"Start a workflow manually." },
  { type:"WEBHOOK_TRIGGER", label:"Webhook Trigger", group:"Triggers", icon:Webhook, tone:"blue", description:"Start when a webhook is received." },
  { type:"SCHEDULE_TRIGGER", label:"Schedule Trigger", group:"Triggers", icon:Clock3, tone:"gold", description:"Start on a recurring schedule." },
  { type:"HTTP_REQUEST", label:"HTTP Request", group:"Actions", icon:Globe2, tone:"blue", description:"Call an external HTTP API." },
  { type:"IF", label:"IF / Condition", group:"Logic", icon:GitBranch, tone:"pink", description:"Branch using a condition." },
  { type:"SET", label:"Set / Transform", group:"Data", icon:Zap, tone:"purple", description:"Create or transform variables." },
  { type:"WHATSAPP_MESSAGE", label:"WhatsApp Message", group:"Actions", icon:MessageSquare, tone:"pink", description:"Send a WhatsApp message." },
  { type:"DELAY", label:"Delay", group:"Logic", icon:Clock3, tone:"gold", description:"Pause execution for a period." },
  { type:"APPROVAL", label:"Approval", group:"Human", icon:Check, tone:"purple", description:"Pause until an authorized user approves." },
];

function makeNode(def, index=0) {
  const cols=4, x=70+(index%cols)*230, y=70+Math.floor(index/cols)*150;
  const configs={
    HTTP_REQUEST:{method:"GET",url:"https://api.goldetech.com/health",headers:{},body:{}},
    IF:{path:"trigger.source",operator:"exists"},
    SET:{values:{source:"GrowthOS"}},
    DELAY:{seconds:2},
    WHATSAPP_MESSAGE:{to:"{{contact.phone}}",content:"Hello {{contact.name}}"},
  };
  return {id:"node_"+Date.now()+"_"+index+"_"+Math.random().toString(36).slice(2,7),node_key:"node_"+(index+1),node_type:def.type,name:def.label,config:configs[def.type]||{},position:{x,y}};
}

function AutomationCanvas({ onBack, onSaved }) {
  const [name,setName]=useState("New Growth Workflow");
  const [description,setDescription]=useState("Built with the GOLD-e GrowthOS workflow canvas.");
  const [nodes,setNodes]=useState(()=>[makeNode(AUTOMATION_NODE_TYPES[0],0)]);
  const [edges,setEdges]=useState([]);
  const [selected,setSelected]=useState(null);
  const [connecting,setConnecting]=useState(null);
  const [connectingOutcome,setConnectingOutcome]=useState(null);
  const [zoom,setZoom]=useState(1);
  const [saving,setSaving]=useState(false);
  const [notice,setNotice]=useState("");
  const [error,setError]=useState("");
  const [createdId,setCreatedId]=useState(null);

  const selectedNode=nodes.find(n=>n.id===selected)||null;
  const nodeDef=(type)=>AUTOMATION_NODE_TYPES.find(n=>n.type===type)||AUTOMATION_NODE_TYPES[0];

  const addNode=(def)=>{
    const n=makeNode(def,nodes.length);
    setNodes(prev=>[...prev,n]);
    setSelected(n.id);
  };
  const updateNode=(id,patch)=>setNodes(prev=>prev.map(n=>n.id===id?{...n,...patch}:n));
  const updateConfig=(id,key,value)=>setNodes(prev=>prev.map(n=>n.id===id?{...n,config:{...n.config,[key]:value}}:n));
  const updateSetValue=(id,key,value)=>setNodes(prev=>prev.map(n=>n.id===id?{...n,config:{...n.config,values:{...(n.config?.values||{}),[key]:value}}}:n));
  const deleteNode=(id)=>{
    setNodes(prev=>prev.filter(n=>n.id!==id));
    setEdges(prev=>prev.filter(e=>e.source!==id&&e.target!==id));
    setSelected(null);setConnecting(null);
  };
  const connectNode=(targetId)=>{
    if(!connecting||connecting===targetId)return;
    const sourceNode=nodes.find(n=>n.id===connecting);
    const config=sourceNode?.node_type==="APPROVAL"&&connectingOutcome
      ?{path:"approval.outcome",operator:"eq",value:connectingOutcome}:{};
    if(edges.some(e=>e.source===connecting&&e.target===targetId&&JSON.stringify(e.config||{})===JSON.stringify(config))){setConnecting(null);setConnectingOutcome(null);return;}
    setEdges(prev=>[...prev,{id:"edge_"+Date.now(),source:connecting,target:targetId,config}]);
    setConnecting(null);setConnectingOutcome(null);
  };
  const startConnection=(nodeId,outcome=null)=>{
    setConnecting(nodeId);setConnectingOutcome(outcome);setSelected(nodeId);
  };
  const removeEdge=(edgeId)=>setEdges(prev=>prev.filter(e=>e.id!==edgeId));
  const edgePoints=(edge)=>{
    const a=nodes.find(n=>n.id===edge.source),b=nodes.find(n=>n.id===edge.target);
    if(!a||!b)return null;
    const x1=a.position.x+184,y1=a.position.y+54,x2=b.position.x,y2=b.position.y+54,dx=Math.max(55,Math.abs(x2-x1)*.45);
    return {x1,y1,x2,y2,d:"M "+x1+" "+y1+" C "+(x1+dx)+" "+y1+", "+(x2-dx)+" "+y2+", "+x2+" "+y2};
  };

  const saveWorkflow=async(publish=false)=>{
    setSaving(true);setError("");setNotice("");
    try{
      const triggerNodes=nodes.filter(n=>n.node_type.endsWith("TRIGGER"));
      if(!name.trim())throw new Error("Workflow name is required.");
      if(!triggerNodes.length)throw new Error("Add at least one trigger node.");
      const triggerTypes=[...new Set(triggerNodes.map(n=>n.node_type))];
      const payload={
        name:name.trim(),description:description.trim()||null,
        triggers:triggerTypes.map(trigger_type=>({trigger_type,config:{},enabled:true})),
        nodes:nodes.map(n=>({node_key:n.node_key,node_type:n.node_type,name:n.name,config:n.config||{},position:n.position||{}})),
        edges:edges.map(e=>({source_node_key:nodes.find(n=>n.id===e.source)?.node_key||"",target_node_key:nodes.find(n=>n.id===e.target)?.node_key||"",config:e.config||{}})),
        schedule:triggerTypes.includes("SCHEDULE_TRIGGER")?{interval_seconds:300,enabled:true}:null
      };
      let id=createdId;
      if(!id){
        const data=unwrap(await automationsApi.create(payload));
        id=data?.id;
        if(!id)throw new Error("Workflow was created but no automation ID was returned.");
        setCreatedId(id);
      }
      if(publish){await automationsApi.publish(id);setNotice("Workflow published to the automation engine.");}
      else setNotice("Workflow saved as a draft.");
      onSaved?.();
    }catch(e){setError(e?.response?.data?.error||e?.message||"Unable to save workflow.")}
    finally{setSaving(false);setTimeout(()=>setNotice(""),3500);}
  };

  const onNodeDragStart=(event,nodeId)=>{
    event.dataTransfer.setData("text/golde-node",nodeId);
    event.dataTransfer.effectAllowed="move";
  };
  const onCanvasDrop=(event)=>{
    event.preventDefault();
    const id=event.dataTransfer.getData("text/golde-node"),n=nodes.find(x=>x.id===id);
    if(!n)return;
    const rect=event.currentTarget.getBoundingClientRect();
    const x=Math.max(20,(event.clientX-rect.left)/zoom-20),y=Math.max(20,(event.clientY-rect.top)/zoom-20);
    updateNode(id,{position:{x,y}});
  };
  const groups=[...new Set(AUTOMATION_NODE_TYPES.map(n=>n.group))];

  return <div className="automation-builder">
    <div className="builder-topbar">
      <div className="builder-title">
        <button className="ghost-btn compact" onClick={onBack}><ChevronLeft size={15}/>Automations</button>
        <div><div className="panel-kicker">WORKFLOW BUILDER</div><strong>{name}</strong></div>
      </div>
      <div className="builder-actions">
        <button className="ghost-btn compact" onClick={()=>setZoom(z=>Math.max(.65,z-.1))}><ZoomOut size={14}/></button>
        <span className="zoom-value">{Math.round(zoom*100)}%</span>
        <button className="ghost-btn compact" onClick={()=>setZoom(z=>Math.min(1.5,z+.1))}><ZoomIn size={14}/></button>
        <button className="ghost-btn compact" onClick={()=>setZoom(1)}><Maximize2 size={14}/></button>
        <button className="ghost-btn compact" disabled={saving} onClick={()=>saveWorkflow(false)}><Save size={14}/>{saving?"Saving…":"Save draft"}</button>
        <button className="command-btn compact" disabled={saving} onClick={()=>saveWorkflow(true)}><Play size={14}/>{saving?"Working…":"Publish"}</button>
      </div>
    </div>
    {(notice||error)&&<div className={"builder-notice "+(error?"error":"")}>{error||notice}</div>}
    <div className="builder-layout">
      <aside className="node-library">
        <div className="library-head"><div><span className="panel-kicker">NODE LIBRARY</span><h3>Build your workflow</h3></div></div>
        <p className="library-help">Drag nodes onto the canvas, then connect them by selecting a source and clicking the target.</p>
        {groups.map(group=><div className="node-group" key={group}>
          <span>{group}</span>
          {AUTOMATION_NODE_TYPES.filter(n=>n.group===group).map(def=>{
            const I=def.icon;
            return <button className="library-node" key={def.type} draggable onDragStart={e=>{const temp=makeNode(def,nodes.length);setNodes(prev=>[...prev,temp]);e.dataTransfer.setData("text/golde-node",temp.id)}} onClick={()=>addNode(def)}>
              <i className={"node-tone "+def.tone}><I size={15}/></i><span><strong>{def.label}</strong><small>{def.description}</small></span><Plus size={14}/>
            </button>
          })}
        </div>)}
      </aside>

      <section className="builder-canvas-wrap">
        <div className="canvas-toolbar"><div className="canvas-hint">{connecting?"Select a target node to create a connection"+(connectingOutcome?" for "+connectingOutcome:"")+".":"Select a node to edit it. Use Connect in the node header to wire the workflow."}</div><div className="canvas-stats"><span>{nodes.length} nodes</span><span>{edges.length} connections</span></div></div>
        <div className="workflow-canvas" onDragOver={e=>e.preventDefault()} onDrop={onCanvasDrop}>
          <div className="canvas-grid" style={{transform:"scale("+zoom+")",transformOrigin:"0 0"}}>
            <svg className="edge-layer" width="2000" height="1400">
              <defs><marker id="golde-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#8b63e8"/></marker></defs>
              {edges.map(edge=>{const p=edgePoints(edge);const label=edge.config?.value==="approved"?"✓ Success":edge.config?.value==="declined"?"✕ Declined":"";return p&&<g key={edge.id} className="edge-line" onClick={()=>removeEdge(edge.id)}><path d={p.d} fill="none" stroke={edge.config?.value==="declined"?"#e24f68":"#9b7be8"} strokeWidth="2.5" markerEnd="url(#golde-arrow)"/><rect x={(p.x1+p.x2)/2-34} y={(p.y1+p.y2)/2-20} width="68" height="18" rx="9" fill="#fff" stroke="#e2d9f3"/><text x={(p.x1+p.x2)/2} y={(p.y1+p.y2)/2-8} textAnchor="middle" fontSize="8" fontWeight="700" fill={edge.config?.value==="declined"?"#c93854":"#6f43cf"}>{label||"× remove"}</text></g>})}
            </svg>
            {nodes.map(node=>{
              const def=nodeDef(node.node_type),I=def.icon;
              return <div key={node.id} className={"workflow-node "+(selected===node.id?"selected ":"")+(connecting===node.id?"connecting":"")} style={{left:node.position.x,top:node.position.y}} draggable onDragStart={e=>onNodeDragStart(e,node.id)} onClick={()=>connecting&&connecting!==node.id?connectNode(node.id):setSelected(node.id)}>
                <div className="workflow-node-head"><i className={"node-tone "+def.tone}><I size={14}/></i><span>{node.node_type}</span><GripVertical size={14}/></div>
                <strong>{node.name}</strong><small>{node.node_type==="APPROVAL"?"Human approval required":node.node_type==="IF"?"Branch on condition":node.config?.url||node.config?.content||def.description}</small>
                <div className="node-connect-row">
  {node.node_type==="APPROVAL"?<>
    <button onClick={e=>{e.stopPropagation();startConnection(node.id,"approved")}} className="approval-connect approved"><Check size={11}/>Success</button>
    <button onClick={e=>{e.stopPropagation();startConnection(node.id,"declined")}} className="approval-connect declined"><X size={11}/>Declined</button>
  </>:<button onClick={e=>{e.stopPropagation();startConnection(node.id)}}><ArrowRight size={12}/>{connecting===node.id?"Connecting…":"Connect"}</button>}
</div>
                <span className="node-input"/><span className="node-output"/>
              </div>
            })}
          </div>
          {nodes.length===0&&<div className="canvas-empty"><Zap size={26}/><strong>Drop your first node here</strong><span>Start with a trigger from the library.</span></div>}
        </div>
      </section>

      <aside className="node-inspector">
        {!selectedNode?<div className="inspector-empty"><PanelRight size={26}/><strong>Select a node</strong><span>Node configuration appears here.</span></div>:
          <div>
            <div className="inspector-head"><div><span className="panel-kicker">CONFIGURATION</span><h3>{selectedNode.name}</h3></div><button className="icon-btn" onClick={()=>deleteNode(selectedNode.id)}><X size={16}/></button></div>
            <label className="inspector-label">Node name<input value={selectedNode.name} onChange={e=>updateNode(selectedNode.id,{name:e.target.value})}/></label>
            {selectedNode.node_type==="HTTP_REQUEST"&&<>
              <label className="inspector-label">Method<select value={selectedNode.config?.method||"GET"} onChange={e=>updateConfig(selectedNode.id,"method",e.target.value)}><option>GET</option><option>POST</option><option>PUT</option><option>PATCH</option><option>DELETE</option></select></label>
              <label className="inspector-label">URL<input value={selectedNode.config?.url||""} onChange={e=>updateConfig(selectedNode.id,"url",e.target.value)} placeholder="https://example.com/api"/></label>
            </>}
            {selectedNode.node_type==="WHATSAPP_MESSAGE"&&<>
              <label className="inspector-label">Recipient<input value={selectedNode.config?.to||""} onChange={e=>updateConfig(selectedNode.id,"to",e.target.value)} placeholder="{{contact.phone}}"/></label>
              <label className="inspector-label">Message<textarea value={selectedNode.config?.content||""} onChange={e=>updateConfig(selectedNode.id,"content",e.target.value)} placeholder="Hello {{contact.name}}"/></label>
            </>}
            {selectedNode.node_type==="DELAY"&&<label className="inspector-label">Seconds<input type="number" min="0" max="300" value={selectedNode.config?.seconds??2} onChange={e=>updateConfig(selectedNode.id,"seconds",Number(e.target.value))}/></label>}
            {selectedNode.node_type==="IF"&&<>
              <label className="inspector-label">Variable path<input value={selectedNode.config?.path||""} onChange={e=>updateConfig(selectedNode.id,"path",e.target.value)} placeholder="trigger.source"/></label>
              <label className="inspector-label">Operator<select value={selectedNode.config?.operator||"exists"} onChange={e=>updateConfig(selectedNode.id,"operator",e.target.value)}><option value="exists">exists</option><option value="eq">equals</option><option value="neq">not equals</option><option value="truthy">truthy</option></select></label>
              {(selectedNode.config?.operator==="eq"||selectedNode.config?.operator==="neq")&&<label className="inspector-label">Value<input value={selectedNode.config?.value??""} onChange={e=>updateConfig(selectedNode.id,"value",e.target.value)}/></label>}
            </>}
            {selectedNode.node_type==="SET"&&<label className="inspector-label">Variable<input value={Object.keys(selectedNode.config?.values||{})[0]||"source"} onChange={e=>{const old=Object.keys(selectedNode.config?.values||{})[0]||"source";const val=selectedNode.config?.values?.[old]||"";updateNode(selectedNode.id,{config:{...selectedNode.config,values:{[e.target.value]:val}}})}}/><textarea value={Object.values(selectedNode.config?.values||{})[0]||""} onChange={e=>{const key=Object.keys(selectedNode.config?.values||{})[0]||"source";updateSetValue(selectedNode.id,key,e.target.value)}} placeholder="Value or {{variable}}"/></label>}
            {selectedNode.node_type.endsWith("TRIGGER")&&<div className="inspector-note">This node creates the corresponding enabled trigger when the workflow is saved.</div>}
            {selectedNode.node_type==="APPROVAL"&&<div className="inspector-note">Execution pauses here. Connect the <b>Success</b> output and <b>Declined</b> output to different downstream nodes.</div>}
            {selectedNode.node_type==="APPROVAL"?<div className="approval-connect-panel">
  <button className="command-btn compact full-btn" onClick={()=>startConnection(selectedNode.id,"approved")}><Check size={14}/>Connect Success path</button>
  <button className="ghost-btn compact full-btn" onClick={()=>startConnection(selectedNode.id,"declined")}><X size={14}/>Connect Declined path</button>
</div>:<button className="command-btn compact full-btn" onClick={()=>startConnection(selectedNode.id)}><ArrowRight size={14}/>{connecting===selectedNode.id?"Select target node":"Connect to next node"}</button>}
          </div>}
      </aside>
    </div>
  </div>;
}

function AutomationsPage() {
  const [items,setItems]=useState([]),[loading,setLoading]=useState(true),[notice,setNotice]=useState("");
  const [selectedRun,setSelectedRun]=useState(null),[runLoading,setRunLoading]=useState(false),[runError,setRunError]=useState(""),[approving,setApproving]=useState(false);
  const [builder,setBuilder]=useState(false);

  const load=()=>{setLoading(true);automationsApi.list().then(unwrap).then(x=>setItems(Array.isArray(x?.data)?x.data:Array.isArray(x)?x:[])).catch(()=>setItems([])).finally(()=>setLoading(false))};
  useEffect(()=>{load()},[]);

  const act=async(id,action)=>{try{await automationsApi[action](id);setNotice("Automation published.");load();setTimeout(()=>setNotice(""),2500)}catch(e){setNotice(e?.response?.data?.error||e?.message||"Action failed.")}};
  const openRun=async(runId)=>{if(!runId)return;setRunLoading(true);setRunError("");try{setSelectedRun(unwrap(await automationsApi.getRun(runId)))}catch(e){setRunError(e?.response?.data?.error||e?.message||"Unable to load automation run.")}finally{setRunLoading(false)}};
  const runAutomation=async(id)=>{setNotice("");try{const data=unwrap(await automationsApi.run(id,{payload:{source:"GrowthOS manual test"}}));const runId=data?.run_id;if(!runId)throw new Error("Automation started but no run ID was returned.");setNotice("Automation run started.");await openRun(runId);setTimeout(()=>setNotice(""),2500)}catch(e){setNotice(e?.response?.data?.error||e?.message||"Automation run failed.")}};
  const approveStep=async(step)=>{if(!selectedRun?.id||!step?.id)return;setApproving(true);setRunError("");try{await automationsApi.approveStep(selectedRun.id,step.id);await openRun(selectedRun.id);setNotice("Approval accepted. Success path resumed.");setTimeout(()=>setNotice(""),2500)}catch(e){setRunError(e?.response?.data?.error||e?.message||"Approval failed.")}finally{setApproving(false)}};
  const declineStep=async(step)=>{if(!selectedRun?.id||!step?.id)return;setApproving(true);setRunError("");try{await automationsApi.declineStep(selectedRun.id,step.id);await openRun(selectedRun.id);setNotice("Approval declined. Declined path resumed.");setTimeout(()=>setNotice(""),2500)}catch(e){setRunError(e?.response?.data?.error||e?.message||"Decline failed.")}finally{setApproving(false)}};
  const steps=Array.isArray(selectedRun?.steps)?selectedRun.steps:[];

  if(builder)return <AutomationCanvas onBack={()=>{setBuilder(false);load()}} onSaved={load}/>;

  return <div>
    <PageHeader eyebrow="AUTOMATION ENGINE" title="Automations" description="Design, test and publish native workflows inside GOLD-e GrowthOS." action={<button className="command-btn" onClick={()=>setBuilder(true)}><Plus size={16}/>Create workflow</button>}/>
    {notice&&<div className="action-notice"><Check size={15}/>{notice}</div>}
    <section className="automation-hero panel"><div><span className="panel-kicker">VISUAL WORKFLOW ENGINE</span><h2>Build automations on a canvas.</h2><p>Drag triggers and actions, connect the flow, configure each node, then save or publish directly to the Rust automation engine.</p></div><button className="primary-btn compact" onClick={()=>setBuilder(true)}><Zap size={15}/>Open workflow builder</button></section>
    <section className="panel table-panel">
      {loading?<div className="empty-state"><div className="spinner"/><h3>Loading automations</h3></div>
      :items.length===0?<div className="empty-state"><div className="empty-icon"><Zap/></div><h3>No automations yet.</h3><p>Create your first workflow with triggers, HTTP actions, conditions and approval gates.</p><button className="command-btn" onClick={()=>setBuilder(true)}><Plus size={16}/>Build first workflow</button></div>
      :<div className="data-list">{items.map(a=><div className="data-row" key={a.id}><div className="row-icon"><Zap size={17}/></div><div><strong>{a.name}</strong><small>{a.status} · v{a.version||1}</small></div><span className="status-pill">{a.status}</span>{a.status!=="PUBLISHED"?<button className="row-action" onClick={()=>act(a.id,"publish")}><Play size={13}/></button>:<span className="row-action-spacer" title="Pause is not available in the current API."/>}<button className="row-action" onClick={()=>runAutomation(a.id)}>Run</button></div>)}</div>}
    </section>
    {(runLoading||selectedRun||runError)&&<div className="run-modal-backdrop" onClick={()=>setSelectedRun(null)}><section className="run-modal" onClick={e=>e.stopPropagation()}>
      <div className="run-modal-head"><div><div className="panel-kicker">AUTOMATION RUN</div><h2>Run details</h2></div><button className="icon-btn" onClick={()=>setSelectedRun(null)} aria-label="Close"><X size={18}/></button></div>
      {runLoading?<div className="empty-state compact"><div className="spinner"/><h3>Loading run…</h3></div>:runError?<div className="alert error">{runError}</div>:selectedRun&&<>
        <div className="run-summary"><div><span>Status</span><strong className={selectedRun.status==="COMPLETED"?"run-success":selectedRun.status==="WAITING_APPROVAL"?"run-waiting":""}>{selectedRun.status||"UNKNOWN"}</strong></div><div><span>Run ID</span><strong>{selectedRun.id||"—"}</strong></div></div>
        <div className="run-steps"><div className="panel-head"><div><div className="panel-kicker">EXECUTION</div><h3>Steps</h3></div><button className="ghost-btn compact" onClick={()=>openRun(selectedRun.id)}>Refresh</button></div>
        {steps.length===0?<div className="run-empty">No execution steps returned.</div>:steps.map((step,index)=><div className="run-step" key={step.id||step.node_key||index}><div className="run-step-number">{index+1}</div><div className="run-step-main"><strong>{step.node_key||step.node_type||"Step"}</strong><small>{step.node_type||"Automation step"}</small></div><span className={"run-step-status status-"+String(step.status||"").toLowerCase()}>{step.status||"UNKNOWN"}</span>{step.status==="WAITING_APPROVAL"&&<div className="run-approval-actions"><button className="command-btn compact" disabled={approving} onClick={()=>approveStep(step)}>{approving?"Working…":"✓ Approve"}</button><button className="ghost-btn compact decline-btn" disabled={approving} onClick={()=>declineStep(step)}>✕ Decline</button></div>}</div>)}</div>
      </>}
    </section></div>}
  </div>;
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
