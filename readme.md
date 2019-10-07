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
        - [x] db entity
        - [ ] update_user
        - [x] delete_user
        - [x] create_user
    - graphql
        - [x] jwt
        - [x] User::get_user(username || user_id)
        - [ ] User::createUser(new_user: NewUser)
        - [ ] User::hash_isValid(has)
        - [ ] validate password
    <!-- - [ ] user class caller (make it easier to find out if is valid() can post() user exist() maybe eeven get_user() and protect that) -->

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
        - [ ] shadow delete
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

## Permssions

what is the best way for permissions.  
have them attatched as flags on the entites?  
have a permission schema?

