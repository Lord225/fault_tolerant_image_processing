CREATE TYPE status_type AS ENUM('pending', 'running', 'completed', 'failed');

CREATE TABLE tasks (
    id         BIGSERIAL NOT NULL PRIMARY KEY,
    task_id    BIGINT NOT NULL, 
    status     status_type NOT NULL,
    timestamp  BIGINT NOT NULL,
    data       VARCHAR(255),
    params     VARCHAR(1024) NOT NULL 
);

CREATE TABLE parents (
    task_id    BIGINT NOT NULL,
    parent_id  BIGINT NOT NULL,
    PRIMARY KEY (task_id, parent_id)
);

-- sequential number for task_id
CREATE SEQUENCE task_id_seq AS BIGINT;