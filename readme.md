# Tolerate random errors in multithreaded image procesing pipeline using journoal recovery 
This App uses database to track pipeline progression and reverts state if any error occurs - including - worker thread panic, missing files, job timeouts.

Tasks are tree-like structures that produce one output and can take multiple inputs (or none for input leafs). 
```
Input1 Input2
  |      |
 Crop    |
  \      /
   \    /
  Overlay
```
Output of each node is stored in temp folder.
Progress of each job is tracked using database and can be resumed, revert or retired at any point.   

## Tracking convention
Each job is tracked using an entry in `tasks` table. Additionally there is an table for storing parent tasks to track if task can be started.
```
| ID | task_id | status  | timestamp | data_path? | job_params
```
```
| task_id | parent_id |
```

Everything is builded with fault tolerance in mind.

# Workers
Engine uses two woerkers - each can perform diffrent operations like Crop, Resize or Brightness. 

# Frontend
Fontend app were builded to better visualize processes. It is built with `iced`. And shows progress of jobs, allows you to add new jobs to tree and alter simulation settings (like throttle and error chance). 

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

Selects all latest statuses for each task.
```SQL
SELECT t.task_id, t.status
FROM tasks t
INNER JOIN (
    SELECT task_id, MAX(id) AS max_id
    FROM tasks
    GROUP BY task_id
) latest ON t.task_id = latest.task_id AND t.id = latest.max_id;
```


```SQL
WITH latest_tasks AS (SELECT t.task_id, t.status
FROM tasks t
INNER JOIN (
    SELECT task_id, MAX(id) AS max_id
    FROM tasks
    GROUP BY task_id
) latest ON t.task_id = latest.task_id AND t.id = latest.max_id)
SELECT * FROM latest_tasks lt LEFT JOIN parents p ON lt.task_id=p.task_id LEFT JOIN latest_tasks lt2 ON p.parent_id=lt2.task_id WHERE lt.status IN ('pending', 'failed') GROUP BY lt.task_id HAVING (COUNT(DISTINCT lt2.status) AND MAX(column_name) = 'completed') OR (COUNT(DISTINCT lt2.status));
```


```SQL
WITH latest_tasks AS (                          
    SELECT t.* FROM tasks t                                                    
    INNER JOIN (
        SELECT task_id, MAX(id) AS max_id
        FROM tasks
        GROUP BY task_id ) latest ON t.task_id = latest.task_id AND t.id = latest.max_id
)
SELECT * FROM latest_tasks WHERE task_id IN (                                                               
SELECT lt.task_id                                                                                                     
FROM latest_tasks lt
LEFT JOIN parents p ON lt.task_id = p.task_id
LEFT JOIN latest_tasks lt2 ON p.parent_id = lt2.task_id
WHERE lt.status IN ('pending', 'failed')
GROUP BY lt.task_id
HAVING ( COUNT(DISTINCT lt2.status) = 0 OR (COUNT(DISTINCT lt2.status) = 1 AND MAX(lt2.status) = 'completed' )));
```

MOZE działa^

