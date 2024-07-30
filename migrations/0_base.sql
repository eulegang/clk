create table if not exists Projects(
  id integer primary key autoincrement,
  name varchar(128) not null
);

create table if not exists Entries(
  id integer primary key autoincrement,
  project_id integer not null,
  start integer not null,
  end integer,

  foreign key(project_id) references Projects(id)
)
