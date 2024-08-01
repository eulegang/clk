# CLK

track projects; run reports

## Basics

### Projecs

Add a list of projects to track time against

### Entries

run `clk on <proj>` to log time against a project.

run `clk off` to stop logging time.

Only one project may be active at a time.

## Reports

### Setup

Data is kept in sqlite.

sqlite queries can be kept in `~/.config/clk/reports/<report>.sql` to create a report in csv.

### Extensions

This project adds scalar functions to queries

| name     | description                                            |
|----------|--------------------------------------------------------|
| minutes  | Extracts minutes component from total seconds          |
| hours    | Extracts hours component from total seconds            |
| days     | Extracts days component from total seconds             |
| duration | Creates a minimal string representation "3d 3h 4m 23s" |

### Sample 

`~/.config/sessions/reports/sessions.sql`

this report shows the different entries loged, to which project, on what day and how long.

```sql
select
  name,
  date(start, 'unixepoch', 'localtime') as day,
  duration(end - start) as duration
from Entries 
inner join Projects on Projects.id = Entries.project_id 
where 
  end is not null and
  start between unixepoch('now', '-7 days') and unixepoch()
```

### Advanced

clk loads the `~/.config/clk/ext.sql` file each time the program is run.

This allows the user to extend the data with more tables or even insert values 
(if they don't exist) that may be used in reports.

