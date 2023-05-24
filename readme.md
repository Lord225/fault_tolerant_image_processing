# Notatki

Task -> Resize -> blur -> Crop

Thread1: Resize, Crop
Thread2: Blur, Brightness
Thread3: Add ?


Tabela tasks
```
| ID | TaskID | parent_task? | status | timestamp | data_path? | job_params
```

status
- pending
- running
- completed
- failed

Jeden thread kontoler. Gada z bazą pobiera z niej taski jakie trzeba robić

Succes
```
| 1 | 1 | null | pending | 0.0 | "img/img1.png"
| 2 | 2 | 1    | pending | 0.1 | null

| 3 | 1 | null | running | 0.2 | "img/img1.png"

| 3 | 1 | null | c       | 0.2 | "img/out1.png"
| 2 | 2 | 1    | running | 0.1 | "img/out1.png"
```


dwie tabele
Tabela images
```
| id | path |
| 1  | img/img1.png |
```
Tabela tasks
```
| id | parent_id | image_id | status  | params | timestamp |
| 1  | null      | 1        | pending | null   | 0.0       |
| 2  | 1         | null     | pending | null   | 0.1       |
| 1  | null      | 1        | running | null   | 0.2       |
```
Tabela jobs
```
| id | taks_id | status  | timestamp |
| 1  | 1       | running | 0.2       |
| 1  | 1       | failed  | 0.2       |
```
BLUR -> resize




Tabela parents
| taskID | parent_id |
| 3      | 1         |
| 3      | 2         |

Tabela tasks
```
| ID | TaskID | parent_task? | status  | timestamp | data_path?     | job_params 
| 1  | 1      | null         | pending | 0.0       | "img/img1.png" | { "blur": 0.5 }
| 2  | 2      | 1            | pending | 0.1       | null           | "resize"

| 3  | 1      | null         | running | 0.2       | null           | "blur"
| 4  | 1      | null         | c       | 0.3       | "img/out1.png" | "blur"

| 5  | 2      | 1            | running | 0.4       | null           | "resize"
| 6  | 2      | 1            | fail    | 0.5       | null           | "resize"
| 7  | 2      | 1            | running | 0.6       | null           | "resize"
| 8  | 2      | 1            | c       | 0.6       | "img/out2.png" | "resize"
```

Wersja na kilka parentów
```sql
CREATE TABLE IF NOT EXISTS tasks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id    INTEGER NOT NULL,
    status     TEXT NOT NULL,
    timestamp  REAL NOT NULL,
    data_path  TEXT,
    job_params TEXT
);
CREATE TABLE IF NOT EXISTS parents (
    task_id    INTEGER NOT NULL,
    parent_id  INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks (task_id),
    FOREIGN KEY (parent_id) REFERENCES tasks (task_id)
    PRIMARY KEY (task_id, parent_id)
);
```

Wersja na jednego parenta
```sql
CREATE TABLE IF NOT EXISTS tasks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id    INTEGER NOT NULL,
    parent_id  INTEGER,
    status     TEXT NOT NULL,
    timestamp  REAL NOT NULL,
    data_path  TEXT,
    job_params TEXT
);
```
