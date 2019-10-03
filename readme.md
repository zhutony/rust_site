# site name
# Todo
- [x] database connection
- [ ] login logic
- [ ] jwt
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