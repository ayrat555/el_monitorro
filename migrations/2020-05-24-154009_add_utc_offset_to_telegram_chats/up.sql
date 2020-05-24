ALTER TABLE telegram_chats ADD COLUMN utc_offset_minutes INTEGER CHECK (utc_offset_minutes >= -720 AND utc_offset_minutes <= 840);
