# Listing Projects

## Getting a simple project list

```
$ cloudtruth projects ls
...
```

## Getting more details

```
$ cloudtruth projects ls -v
...
```

# Creating a Project

Use the set command to create a new project:

```
$ cloudtruth projects set [NAME] --desc 'A new project'
Created project '[NAME]'

```

Note: replace [NAME] with your own project name (no brackets needed).

You can now see your new project in the list:

```
$ cloudtruth projects ls -v -f csv
...
[NAME],,A new project
...
```

# Updating a Project

Running again on an existing project will update it.

```
$ cloudtruth projects set [NAME] --desc 'An updated project'
Updated project '[NAME]'

$ cloudtruth projects ls -v -f csv
...
[NAME],,An updated project
...
```

# Renaming a project

Use the --rename option to rename a project:

```
$ cloudtruth projects set [OLD] --rename [NEW]
Updated project '[NEW]'

```

# Show timestamps

You can view create and modify timestamps with the --show-times option:

```
$ cloudtruth projects list --show-times -f csv
Name,Parent,Description,Created At,Modified At
...
[NEW],,Updated description,[..],[..]
...
```

# Deleting a project

```
$ cloudtruth projects delete [NEW] --confirm
Deleted project '[NEW]'

```
