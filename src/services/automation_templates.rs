use serde_json::{json, Value};

pub fn list() -> Value {
    json!([
        {
            "id":"welcome-webhook",
            "name":"Welcome Lead",
            "description":"Webhook → enrich payload → AI qualification → HTTP action.",
            "category":"Lead Management",
            "trigger_type":"WEBHOOK",
            "graph":{
                "nodes":[
                    {"id":"trigger","name":"Lead Webhook","type":"trigger.webhook","config":{},"position":[80,180]},
                    {"id":"set","name":"Normalize Lead","type":"data.set","config":{"data":{"source":"webhook","status":"new"}},"position":[360,180]},
                    {"id":"ai","name":"AI Qualification","type":"ai.agent","config":{"provider":"openai-compatible","endpoint":"","model":"default","prompt":"Qualify this lead and return structured JSON.","api_key_env":"AI_API_KEY"},"position":[640,180]},
                    {"id":"action","name":"CRM Action","type":"http.request","config":{"method":"POST","url":"","body":{"lead":"{{$json}}"}},"position":[920,180]}
                ],
                "edges":[
                    {"source":"trigger","target":"set"},
                    {"source":"set","target":"ai"},
                    {"source":"ai","target":"action"}
                ]
            }
        },
        {
            "id":"lead-routing",
            "name":"Lead Routing with IF",
            "description":"Webhook → score → condition → high/low routing.",
            "category":"Lead Management",
            "trigger_type":"WEBHOOK",
            "graph":{
                "nodes":[
                    {"id":"trigger","name":"Lead Webhook","type":"trigger.webhook","config":{},"position":[80,180]},
                    {"id":"condition","name":"Lead Score Check","type":"logic.condition","config":{"path":"score","equals":80},"position":[400,180]},
                    {"id":"high","name":"High Intent Action","type":"http.request","config":{"method":"POST","url":"","body":{"route":"high","lead":"{{$json}}"}},"position":[720,100]},
                    {"id":"low","name":"Nurture Action","type":"http.request","config":{"method":"POST","url":"","body":{"route":"nurture","lead":"{{$json}}"}},"position":[720,280]}
                ],
                "edges":[
                    {"source":"trigger","target":"condition"},
                    {"source":"condition","target":"high","branch":"true"},
                    {"source":"condition","target":"low","branch":"false"}
                ]
            }
        },
        {
            "id":"scheduled-report",
            "name":"Scheduled Growth Report",
            "description":"Runs on an interval and sends the generated report to an HTTP endpoint.",
            "category":"Reporting",
            "trigger_type":"SCHEDULE",
            "trigger_config":{"interval_seconds":86400},
            "graph":{
                "nodes":[
                    {"id":"trigger","name":"Daily Schedule","type":"trigger.schedule","config":{"interval_seconds":86400},"position":[80,180]},
                    {"id":"ai","name":"Generate Report","type":"ai.agent","config":{"provider":"openai-compatible","endpoint":"","model":"default","prompt":"Create a concise growth report from the input data.","api_key_env":"AI_API_KEY"},"position":[420,180]},
                    {"id":"action","name":"Deliver Report","type":"http.request","config":{"method":"POST","url":"","body":{"report":"{{$json}}"}},"position":[780,180]}
                ],
                "edges":[
                    {"source":"trigger","target":"ai"},
                    {"source":"ai","target":"action"}
                ]
            }
        }
    ])
}

pub fn get(id: &str) -> Option<Value> {
    list().as_array()?.iter().find(|x| x.get("id").and_then(Value::as_str) == Some(id)).cloned()
}
