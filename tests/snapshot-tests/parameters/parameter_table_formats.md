# test_parameter_table_formats

check that there are no parameters

```console
$ cloudtruth parameters list
No parameters found in project [PROJECT]

```

add a couple parameters

```console
$ cloudtruth param set "speicla3" --value "beef brocolli, pork fried rice" --desc "Jade lunch" 
Set parameter 'speicla3' in project '[PROJECT]' for environment 'default'.

$ cloudtruth param set "speicla14" --value "cueey-chicken" --secret "true" --desc "Jade secret" 
Set parameter 'speicla14' in project '[PROJECT]' for environment 'default'.

```

table format

```console
$ cloudtruth parameters ls -v
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| Name      | Value                          | Source  | Param Type | Rules | Type     | Secret | Description |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| speicla14 | *****                          | default | string     | 0     | internal | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | string     | 0     | internal | false  | Jade lunch  |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+

$ cloudtruth parameters ls -v -s
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| Name      | Value                          | Source  | Param Type | Rules | Type     | Secret | Description |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| speicla14 | cueey-chicken                  | default | string     | 0     | internal | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | string     | 0     | internal | false  | Jade lunch  |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+

```

CSV format

```console
$ cloudtruth parameters ls -v -f csv
Name,Value,Source,Param Type,Rules,Type,Secret,Description
speicla14,*****,default,string,0,internal,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,string,0,internal,false,Jade lunch

$ cloudtruth parameters ls -v -f csv -s
Name,Value,Source,Param Type,Rules,Type,Secret,Description
speicla14,cueey-chicken,default,string,0,internal,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,string,0,internal,false,Jade lunch

```

JSON format

```console
$ cloudtruth parameters ls -v -f json
{
  "parameter": [
    {
      "Description": "Jade secret",
      "Name": "speicla14",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "true",
      "Source": "default",
      "Type": "internal",
      "Value": "*****"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "internal",
      "Value": "beef brocolli, pork fried rice"
    }
  ]
}

$ cloudtruth parameters ls -v -f json -s
{
  "parameter": [
    {
      "Description": "Jade secret",
      "Name": "speicla14",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "true",
      "Source": "default",
      "Type": "internal",
      "Value": "cueey-chicken"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "internal",
      "Value": "beef brocolli, pork fried rice"
    }
  ]
}

```

YAML format

```console
$ cloudtruth parameters ls -v -f yaml
---
parameter:
  - Description: Jade secret
    Name: speicla14
    Param Type: string
    Rules: "0"
    Secret: "true"
    Source: default
    Type: internal
    Value: "*****"
  - Description: Jade lunch
    Name: speicla3
    Param Type: string
    Rules: "0"
    Secret: "false"
    Source: default
    Type: internal
    Value: "beef brocolli, pork fried rice"

$ cloudtruth parameters ls -v -f yaml -s
---
parameter:
  - Description: Jade secret
    Name: speicla14
    Param Type: string
    Rules: "0"
    Secret: "true"
    Source: default
    Type: internal
    Value: cueey-chicken
  - Description: Jade lunch
    Name: speicla3
    Param Type: string
    Rules: "0"
    Secret: "false"
    Source: default
    Type: internal
    Value: "beef brocolli, pork fried rice"

```
