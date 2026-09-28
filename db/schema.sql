create table if not exists chain_cursor (
  name text primary key,
  slot bigint not null,
  signature text,
  updated_at timestamptz not null default now()
);

create table if not exists market_events (
  signature text not null,
  slot bigint not null,
  log_index integer not null,
  program_id text not null,
  event_kind text not null,
  payload jsonb not null,
  finalized boolean not null default false,
  block_time bigint,
  primary key(signature, log_index)
);

create index if not exists market_events_slot_idx on market_events(slot);
create index if not exists market_events_kind_idx on market_events(event_kind);
