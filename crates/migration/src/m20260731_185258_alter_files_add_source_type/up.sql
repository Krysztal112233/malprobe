-- 来源类型：url（下载地址）| upload（直接上传的字节，future）
CREATE TYPE file_source_type AS ENUM ('url', 'upload');

-- 泛化来源列：url → 下载地址；upload → 原始文件名（可选）
ALTER TABLE files RENAME COLUMN source_url TO source;
ALTER TABLE files ALTER COLUMN source DROP NOT NULL;
ALTER TABLE files ADD COLUMN source_type file_source_type NOT NULL DEFAULT 'url';
