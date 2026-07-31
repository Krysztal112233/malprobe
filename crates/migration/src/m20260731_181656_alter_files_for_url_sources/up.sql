-- sha256/size 在上传时未知，worker 下载文件后回填
ALTER TABLE files ALTER COLUMN sha256 DROP NOT NULL;
ALTER TABLE files ALTER COLUMN size DROP NOT NULL;

-- URL 模式取代本地存储路径：backend 只记录下载地址，worker 自行拉取
ALTER TABLE files DROP COLUMN storage_path;
ALTER TABLE files ADD COLUMN source_url TEXT NOT NULL DEFAULT '';
ALTER TABLE files ALTER COLUMN source_url DROP DEFAULT;
