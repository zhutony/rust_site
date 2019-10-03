# site name
# Todo

## Frontentd
- [ ] flutter app
    - [ ] login
    - [ ] posts

## Backend
- [ ] graphql
    - [x] serve ( actix web )
    - [ ] subscriptions ( not implemented in juniper yet)

- [ ] database connection
    - [x] pooled
    - [x] graphql.context
    - [x] sqlite demo
    - [ ] postgresql

- [ ] user.login
- [ ] posts

### Entities

- [ ] User entity
    - db
        - [ ] db entity
        - [ ] update_user
        - [ ] delete_user
        - [ ] create_user
    - graphql
        - [x] jwt
        - [ ] username. pass_hash
        - [ ] isValid

- [ ] post entity
    - db
        - [ ] update
        - [x] get_posts
        - [x] get_post(id || parent_id)
        - [x] get_recursive -sql
    - graphql
        - [x] post (id || parent_id || null)
        - [x] posts (parent_id || null)
        - [x] create_post(parent_id, content)
        - [x] delete_post
        - [x] delete_post_recursive
        - [ s] shadow delete
        - [x] errors

## Site purpose
### application layout


```yaml

front-end:
    - authentication:
        - login
        - logout
    - s3 bucket upload -> backend record

backend:
    nginx :
        - http redirect :
            - http proxy (@app)

    app:
        - http:
            - /
            - /graphql
        - db
        - graphql:
            - posts

    content_management


```

### File layout


```yaml
src:
    main:
    graphql:
        - context
        - schema
        - posts


# vs

src:
    main:
    graphql:
        - context
        - schema
        - posts
    models:
        - posts


```