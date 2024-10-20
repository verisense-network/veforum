


## Model

```
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct VeSubspace {
    pub id: u64,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub banner: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct VeArticle {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: String,
    pub subspace_id: u64,
    pub extlink: String,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
    pub updated_time: i64,
}

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct VeComment {
    pub id: u64,
    pub content: String,
    pub author_id: u64,
    pub author_nickname: u64,
    pub post_id: u64,
    pub status: i16,
    pub weight: i16,
    pub created_time: i64,
}
```


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
