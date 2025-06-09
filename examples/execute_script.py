"""
A short example showing how to execute a script containing a bunch of sql statements.
"""
import os

import libsql

def execute_script(conn, file_path: os.PathLike):
    with open(file_path, 'r') as file:
        script = file.read()

    conn.executescript(script)
    conn.commit()

conn = libsql.connect(':memory:')
script_path = os.path.join(os.path.dirname(__file__), 'statements.sql')
execute_script(conn, script_path)

# Retrieve the data from the 'users' table and print it
cursor = conn.cursor()
cursor.execute("SELECT * FROM users")
rows = cursor.fetchall()
print("Data in the 'users' table:")
for row in rows:
    print(row)
