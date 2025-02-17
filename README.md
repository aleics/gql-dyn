# gql-dyn

This is a proof of concept for building GraphQL schemas dynamically using [juniper](https://github.com/graphql-rust/juniper).

## Start
To run the project, use cargo:

```shell
cargo run
```

This will spin up a [graphiql](https://github.com/graphql/graphiql) client, which can be accessed in your browser at `http://127.0.0.1:8080`

## Motivation
Generating a dynamic GraphQL API based on the user's context is an advanced use case, but useful in certain scenarios. There's not a clear path on how to implement this type of customization using *juniper*. Thus, I wanted to explore how/if it would be possible.

## Approach
This repository demonstrates one possible approach by leveraging the existing [`GraphQLValue::TypeInfo`](https://docs.rs/juniper/latest/juniper/trait.GraphQLValue.html#associatedtype.TypeInfo) to provide a custom configuration during type generation. This configuration specifies which types and fields are available, and these are used to generate the final GraphQL type dynamically.
