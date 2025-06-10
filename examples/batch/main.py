import libsql

conn = libsql.connect("local.db")
cur = conn.cursor()

cur.executescript(
    """
        DROP TABLE IF EXISTS users;
        CREATE TABLE users (id INTEGER, name TEXT);
        INSERT INTO users VALUES (1, 'first@example.org');
        INSERT INTO users VALUES (2, 'second@example.org');
        INSERT INTO users VALUES (3, 'third@example.org');
    """
)

print(conn.execute("select * from users").fetchall())
