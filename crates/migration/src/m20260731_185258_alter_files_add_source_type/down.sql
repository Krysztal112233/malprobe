ALTER TABLE files DROP COLUMN source_type;
ALTER TABLE files ALTER COLUMN source SET NOT NULL;
ALTER TABLE files RENAME COLUMN source TO source_url;
DROP TYPE file_source_type;
