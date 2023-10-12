
## Edit config/Settings.toml
```toml
api_key = "yourapikey"
```

## test and dev functions;

```shell
# postdata;
curl -XPOST -H 'x-api-key:yourapikey' -H 'Content-Type:application/json' http://127.0.0.1:8030/api/login -d '{"username": "foxx","password": "doxx","ipaddress": "0.0.0.0"}'
```
```shell
# get data
curl -XGET -H 'x-api-key:yourapikey' http://127.0.0.1:8030/api/<session_id>
```
