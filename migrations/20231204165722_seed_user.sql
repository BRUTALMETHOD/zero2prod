-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    'ddf8994f-d522-4659-8d02-c1d479057abc',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$4qDOGVqyYZfPgXoIsMYwCg$dZLIepDKbW8O+CEVNk1dcN6lMOR+JqWRer+c05zJckg'
);
