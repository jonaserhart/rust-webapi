-- Your SQL goes here
ALTER TABLE "users"
    ADD COLUMN "email" VARCHAR NOT NULL default '',
    ADD COLUMN "password" VARCHAR NOT NULL default '';
