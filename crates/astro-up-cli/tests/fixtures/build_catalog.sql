-- Test catalog fixture for CLI integration tests.
-- Run: sqlite3 test-catalog.db < build_catalog.sql

CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
INSERT INTO meta VALUES ('schema_version', '1');
INSERT INTO meta VALUES ('compiled_at', '2026-04-04T00:00:00Z');

CREATE TABLE packages (
    id TEXT PRIMARY KEY,
    manifest_version INTEGER DEFAULT 1,
    name TEXT NOT NULL,
    description TEXT,
    publisher TEXT,
    homepage TEXT,
    category TEXT NOT NULL DEFAULT 'other',
    type TEXT NOT NULL DEFAULT 'application',
    slug TEXT,
    license TEXT,
    tags TEXT DEFAULT '[]',
    aliases TEXT DEFAULT '[]',
    dependencies TEXT DEFAULT '[]',
    icon_base64 TEXT
);

CREATE VIRTUAL TABLE packages_fts USING fts5(
    name, description, tags, aliases, publisher,
    content=packages, content_rowid=rowid
);

INSERT INTO packages (id, name, description, publisher, homepage, category, type, slug, license, tags)
VALUES ('test-app', 'Test Application', 'A test application for CI', 'Test Publisher', 'https://example.com', 'capture', 'application', 'test-app', 'MIT', '["test","fixture"]');

INSERT INTO packages (id, name, description, publisher, homepage, category, type, slug, license, tags)
VALUES ('test-driver', 'Test Driver', 'A test ASCOM driver', 'Test Vendor', 'https://example.com', 'driver', 'driver', 'test-driver', 'MIT', '["driver","test"]');

INSERT INTO packages (id, name, description, publisher, homepage, category, type, slug, license, tags)
VALUES ('test-solver', 'Test Plate Solver', 'A test plate solver', 'Solver Co', 'https://example.com', 'plate-solver', 'application', 'test-solver', 'GPL-3.0', '["solver","astrometry"]');

INSERT INTO packages_fts (rowid, name, description, tags, aliases, publisher)
SELECT rowid, name, description, tags, aliases, publisher FROM packages;

CREATE TABLE versions (
    package_id TEXT NOT NULL REFERENCES packages(id),
    version TEXT NOT NULL,
    url TEXT,
    sha256 TEXT,
    discovered_at TEXT DEFAULT (datetime('now')),
    installer_type TEXT DEFAULT 'exe',
    silent_args TEXT DEFAULT '[]',
    PRIMARY KEY (package_id, version)
);

INSERT INTO versions (package_id, version, url, sha256, installer_type, silent_args)
VALUES ('test-app', '2.0.0', 'https://example.com/test-app-2.0.0.exe', 'abc123', 'inno_setup', '["/VERYSILENT","/NORESTART"]');

INSERT INTO versions (package_id, version, url, sha256, installer_type, silent_args)
VALUES ('test-app', '1.0.0', 'https://example.com/test-app-1.0.0.exe', 'def456', 'inno_setup', '["/VERYSILENT","/NORESTART"]');

INSERT INTO versions (package_id, version, url, sha256, installer_type)
VALUES ('test-driver', '3.1.0', 'https://example.com/test-driver-3.1.0.exe', 'ghi789', 'exe');

INSERT INTO versions (package_id, version, url, sha256, installer_type)
VALUES ('test-solver', '1.5.0', 'https://example.com/test-solver-1.5.0.zip', 'jkl012', 'zip');

CREATE TABLE detection (
    package_id TEXT PRIMARY KEY REFERENCES packages(id),
    method TEXT NOT NULL,
    path TEXT,
    registry_key TEXT,
    registry_value TEXT,
    fallback_method TEXT,
    fallback_path TEXT
);

INSERT INTO detection (package_id, method, registry_key, registry_value)
VALUES ('test-app', 'registry', 'HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\TestApp_is1', 'DisplayVersion');

INSERT INTO detection (package_id, method, path)
VALUES ('test-solver', 'file', '{program_files}\TestSolver\solver.exe');
