CREATE TABLE nodes(
    nodepool text,
    hostname text,
    ip inet,
    last_ping bigint,
    created_at bigint,
    PRIMARY KEY (nodepool, hostname)
);

CREATE TABLE rulesets(
    name text,
    rules blob,

    PRIMARY KEY (name)
);