ALTER TABLE telegram_subscriptions ADD COLUMN external_id uuid NOT NULL DEFAULT uuid_generate_v4();
CREATE INDEX telegram_subscriptions_external_id ON telegram_subscriptions(external_id);
