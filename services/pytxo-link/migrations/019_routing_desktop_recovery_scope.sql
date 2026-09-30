-- Recovery codes may renew a Desktop credential only for inspecting and
-- revoking existing grants. An already-applied 017 keeps ordinary scope.
ALTER TABLE routing_desktop_codes
    ADD COLUMN scope TEXT NOT NULL DEFAULT 'routing:grants:v1'
    CHECK (scope IN ('routing:grants:v1', 'routing:revoke:v1'));

ALTER TABLE routing_desktop_sessions
    DROP CONSTRAINT routing_desktop_sessions_scope_check;
ALTER TABLE routing_desktop_sessions
    ADD CONSTRAINT routing_desktop_sessions_scope_check
    CHECK (scope IN ('routing:grants:v1', 'routing:revoke:v1'));
