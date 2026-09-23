-- Seed User (Password: GrowthOS@2026!)
-- Hash: $2b$10$7Zrqh/m1R7V8Bw5i.oP3AOPX8bQ0P4Y4QWp7z9u/hF6L9eYvH8A0e (example bcrypt hash)
INSERT INTO users (id, email, password_hash, first_name, last_name, phone, role, is_active)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'admin@golde-ai.com',
    '$2a$10$wT24.N63K6m8bX2gYVwF3.67Mhz9lI7XkRkS32k8h2G6P7W9N3M4q',
    'Admin',
    'Commander',
    '+919876543210',
    'SYSTEM_ADMIN',
    true
) ON CONFLICT (email) DO NOTHING;

-- Seed Workspace
INSERT INTO workspaces (id, name, slug, description, owner_id, settings)
VALUES (
    'b0000000-0000-0000-0000-000000000001',
    'GOLD-e HQ Growth Engine',
    'golde-hq',
    'Primary enterprise workspace for AI agents and WhatsApp marketing automation',
    'a0000000-0000-0000-0000-000000000001',
    '{"simulationMode": true, "whatsappPhoneId": "100609349493921", "timezone": "Asia/Kolkata"}'::jsonb
) ON CONFLICT (slug) DO NOTHING;

-- Seed Membership
INSERT INTO workspace_members (id, workspace_id, user_id, role)
VALUES (
    'c0000000-0000-0000-0000-000000000001',
    'b0000000-0000-0000-0000-000000000001',
    'a0000000-0000-0000-0000-000000000001',
    'OWNER'
) ON CONFLICT (workspace_id, user_id) DO NOTHING;

-- Seed Contacts
INSERT INTO contacts (id, workspace_id, first_name, last_name, email, phone, company, title, tags, status, custom_fields)
VALUES
(
    'd0000000-0000-0000-0000-000000000001',
    'b0000000-0000-0000-0000-000000000001',
    'Vikram',
    'Singhania',
    'vikram@concreteinfratech.in',
    '+919822011223',
    'Singhania Infra RMC',
    'Managing Director',
    ARRAY['vip', 'high-intent', 'rmc-buyer'],
    'ACTIVE',
    '{"gradeInterest": "M-30, M-40", "monthlyDemandCuM": 3000}'::jsonb
),
(
    'd0000000-0000-0000-0000-000000000002',
    'b0000000-0000-0000-0000-000000000001',
    'Anjali',
    'Patil',
    'anjali.p@patilbuilders.com',
    '+919822011224',
    'Patil Landmark Developers',
    'Procurement Head',
    ARRAY['commercial-builder', 'pune-metro'],
    'ACTIVE',
    '{"fleetSize": 18}'::jsonb
) ON CONFLICT (id) DO NOTHING;

-- Seed Campaign
INSERT INTO campaigns (id, workspace_id, name, description, channel, status, target_tags, template_id, created_by_id)
VALUES (
    'e0000000-0000-0000-0000-000000000001',
    'b0000000-0000-0000-0000-000000000001',
    'Q3 RMC Buyer Reactivation',
    'Automated WhatsApp outreach to dormant contractors',
    'WHATSAPP',
    'RUNNING',
    ARRAY['rmc-buyer'],
    'rmc_monsoon_offer_v2',
    'a0000000-0000-0000-0000-000000000001'
) ON CONFLICT (id) DO NOTHING;

-- Seed Messages
INSERT INTO messages (id, workspace_id, campaign_id, contact_id, channel, direction, external_message_id, status, content)
VALUES
(
    'f0000000-0000-0000-0000-000000000001',
    'b0000000-0000-0000-0000-000000000001',
    'e0000000-0000-0000-0000-000000000001',
    'd0000000-0000-0000-0000-000000000001',
    'WHATSAPP',
    'OUTBOUND',
    'wamid.HBgLOTE5ODIyMDExMjIzFQIAERgSMzEyNDM1NDY1NzY4Nzk4MDFB',
    'DELIVERED',
    'Hello Vikram, explore exclusive monsoon pricing on M-30 and M-40 ready mix concrete.'
),
(
    'f0000000-0000-0000-0000-000000000002',
    'b0000000-0000-0000-0000-000000000001',
    'e0000000-0000-0000-0000-000000000001',
    'd0000000-0000-0000-0000-000000000002',
    'WHATSAPP',
    'OUTBOUND',
    'wamid.HBgLOTE5ODIyMDExMjI0FQIAERgSMzEyNDM1NDY1NzY4Nzk4MDJB',
    'READ',
    'Hello Anjali, Patil Landmark projects can now track live transit mixers via Track My RMC.'
) ON CONFLICT (id) DO NOTHING;

-- Seed Agent Run & Steps (Waiting for Approval)
INSERT INTO agent_runs (id, workspace_id, agent_type, name, status, triggered_by_id, input_params, output_data, started_at)
VALUES (
    '10000000-0000-0000-0000-000000000001',
    'b0000000-0000-0000-0000-000000000001',
    'LEAD_NURTURE_AUTONOMOUS',
    'High-Value Lead Negotiation & WhatsApp Outreach',
    'WAITING_APPROVAL',
    'a0000000-0000-0000-0000-000000000001',
    '{"segment": "Tier-1 Contractors", "budgetCapMinor": 500000}'::jsonb,
    '{"identifiedLeads": 14, "draftReady": true}'::jsonb,
    NOW()
) ON CONFLICT (id) DO NOTHING;

INSERT INTO agent_steps (id, run_id, step_number, name, description, action_type, status, requires_approval, started_at, completed_at)
VALUES (
    '20000000-0000-0000-0000-000000000001',
    '10000000-0000-0000-0000-000000000001',
    1,
    'Analyze Historical Interaction & Credit Limits',
    'Pull payment ledger and site requirements.',
    'DATA_ENRICHMENT',
    'COMPLETED',
    false,
    NOW() - INTERVAL '10 minutes',
    NOW() - INTERVAL '8 minutes'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO agent_steps (id, run_id, step_number, name, description, action_type, status, requires_approval, input_payload, started_at)
VALUES (
    '20000000-0000-0000-0000-000000000002',
    '10000000-0000-0000-0000-000000000001',
    2,
    'Authorize Custom 14% Rebate Broadcast to 14 Key Contractors',
    'Requires executive authorization before triggering bulk WhatsApp dispatch.',
    'APPROVAL_GATEWAY',
    'WAITING_APPROVAL',
    true,
    '{"recipientsCount": 14, "discountTier": "14%", "projectedMargin": "21.5%"}'::jsonb,
    NOW() - INTERVAL '7 minutes'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO agent_steps (id, run_id, step_number, name, description, action_type, status, requires_approval)
VALUES (
    '20000000-0000-0000-0000-000000000003',
    '10000000-0000-0000-0000-000000000001',
    3,
    'Autonomous Dispatch via Meta WhatsApp Cloud API',
    'Send personalized WhatsApp templates to approved recipient list.',
    'WHATSAPP_DISPATCH',
    'PENDING',
    false
) ON CONFLICT (id) DO NOTHING;
