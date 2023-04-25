# test_parameter_secret_switch

check that there are no parameters

```console
$ cloudtruth parameters list --values --secrets
No parameters found in project [PROJECT]

```

add first, non-secret parameter

```console
$ cloudtruth parameters set my_param --value "cRaZy value" --desc "this is just a test description"
Set parameter 'my_param' in project '[PROJECT]' for environment 'default'.

$ cloudtruth parameters ls -v
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | false  | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+

```

switch it to a secret

```console
$ cloudtruth parameters set my_param --secret true
Updated parameter 'my_param' in project '[PROJECT]'.

```

see that it has been changed to a secret (redacted in cli)

```console
$ cloudtruth parameters ls -v
+----------+-------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | ***** | default | string     | 0     | internal | true   | this is just a test description |
+----------+-------+---------+------------+-------+----------+--------+---------------------------------+

```

verify value has not changed

```console
$ cloudtruth parameters ls -v -s
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | true   | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+

```

switch back to a regular parameter

```console
$ cloudtruth parameters set my_param --secret false
Updated parameter 'my_param' in project '[PROJECT]'.

```

see that it is no longer redacted

```console
$ cloudtruth parameters ls -v
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | false  | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+

```
