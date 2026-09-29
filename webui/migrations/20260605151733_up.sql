CREATE TABLE IF NOT EXISTS "printer_queue_membership" (
"uid" uuid NOT NULL PRIMARY KEY,
"printer" uuid NOT NULL,
"queue" uuid NOT NULL,
"active" boolean NOT NULL,
"last_job_at" bigint
);
CREATE TABLE IF NOT EXISTS "printer_discovery_events" (
"uid" uuid NOT NULL PRIMARY KEY,
"mdns_instance" character varying NOT NULL,
"host" character varying NOT NULL,
"port" integer NOT NULL,
"discovered_at" bigint NOT NULL,
"txt" character varying,
"success" boolean NOT NULL,
"error" character varying
);
CREATE TABLE IF NOT EXISTS "print_queues" (
"uid" uuid NOT NULL PRIMARY KEY,
"rp" character varying NOT NULL,
"name" character varying NOT NULL,
"capability" character varying NOT NULL,
"created_at" bigint NOT NULL,
"last_active" bigint,
"advertised" boolean NOT NULL
);
CREATE TABLE IF NOT EXISTS "printers" (
"uid" uuid NOT NULL PRIMARY KEY,
"name" character varying NOT NULL,
"mdns_instance" character varying NOT NULL,
"host" character varying NOT NULL,
"port" integer NOT NULL,
"first_seen" bigint NOT NULL,
"last_seen" bigint NOT NULL,
"last_verified" bigint,
"status" character varying NOT NULL,
"enabled" boolean NOT NULL
);
CREATE TABLE IF NOT EXISTS "print_job_executions" (
"uid" uuid NOT NULL PRIMARY KEY,
"job" uuid NOT NULL,
"printer" uuid NOT NULL,
"attempt" integer NOT NULL,
"state" character varying NOT NULL,
"started_at" bigint,
"finished_at" bigint,
"error" character varying
);
CREATE TABLE IF NOT EXISTS "user_activation_code" (
"uid" uuid NOT NULL PRIMARY KEY,
"user_uid" uuid NOT NULL,
"code" character varying NOT NULL,
"expires_at" bigint NOT NULL,
"used" boolean NOT NULL,
"invalidated" boolean NOT NULL,
"invalidation_reason" character varying
);
CREATE TABLE IF NOT EXISTS "asset" (
"uid" uuid NOT NULL PRIMARY KEY,
"name" character varying NOT NULL,
"mime" character varying NOT NULL,
"owner" uuid NOT NULL,
"size" bigint NOT NULL,
"created_at" bigint NOT NULL,
"updated_at" bigint NOT NULL,
"last_used_at" bigint,
"public" boolean NOT NULL,
"license" character varying
);
CREATE TABLE IF NOT EXISTS "print_jobs" (
"uid" uuid NOT NULL PRIMARY KEY,
"user" uuid NOT NULL,
"queue" uuid NOT NULL,
"title" character varying NOT NULL,
"asset" uuid NOT NULL,
"mime" character varying NOT NULL,
"units" double precision,
"estimated_cost" double precision,
"created_at" bigint NOT NULL,
"state" character varying NOT NULL,
"completed_at" bigint,
"copies" integer,
"priority" character varying,
"material" character varying,
"nozzle_temp" integer,
"bed_temp" integer,
"layer_height" real,
"infill" integer,
"supports" character varying,
"notes" character varying
);