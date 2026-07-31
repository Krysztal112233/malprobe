-- 同一文件可从不同来源多次提交，各自保留独立扫描记录；
-- 去重交给上层业务（如按哈希查询返回列表）
ALTER TABLE files DROP CONSTRAINT files_sha256_key;
