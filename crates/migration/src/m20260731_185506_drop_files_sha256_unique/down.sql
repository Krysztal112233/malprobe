-- 回滚前需确保现有数据无重复 sha256（非空值）
ALTER TABLE files ADD CONSTRAINT files_sha256_key UNIQUE (sha256);
