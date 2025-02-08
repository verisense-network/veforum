


## Model

see vemodel crate.

## API

jsonrpc method call

```
add_subspace(sb: VeSubspace)
update_subspace(sb: VeSubspace)
delete_subspace(id: u64)

add_article(ar: VeArticle)
update_article(ar: VeArticle)
delete_article(id: u64)

add_comment(co: VeComment)
update_comment(co: VeComment)
delete_comment(id: u64)
```


## Test Method

like follows:

### Post

```
$ curl localhost:9944 -H 'Content-Type: application/json' -XPOST -d '{"jsonrpc":"2.0", "id":"whatever", "method":"nucleus_post", "params": ["5FsXfPrUDqq6abYccExCTUxyzjYaaYTr5utLx2wwdBv1m8R8", "add_user", "000000000000000014416c696365"]}'
```

### Get 

```
$ curl localhost:9944 -H 'Content-Type: application/json' -XPOST -d '{"jsonrpc":"2.0", "id":"whatever", "method":"nucleus_get", "params": ["5FsXfPrUDqq6abYccExCTUxyzjYaaYTr5utLx2wwdBv1m8R8", "get_user", "0100000000000000"]}'
```
