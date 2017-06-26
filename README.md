# Rust Web Testing

Hey! I'm developing this web service to help me learn Rust. Please tell me how to make it more idiomatic because I spew `match`es everywhere.

### Current abilities
`POST` to `/sign-up` with a `username` and `password` creates a `User` entry and a `VerificationCode` entry in the database.
`GET` to `/verify/<verification_code>` marks `User` as verified and deletes associated `VerificationCode`.
