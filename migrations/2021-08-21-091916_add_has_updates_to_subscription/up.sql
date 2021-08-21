ALTER TABLE telegram_subscriptions ADD COLUMN has_updates BOOLEAN NOT NULL DEFAULT true;
CREATE INDEX telegram_subscriptions_has_updates_index ON telegram_subscriptions(has_updates);
