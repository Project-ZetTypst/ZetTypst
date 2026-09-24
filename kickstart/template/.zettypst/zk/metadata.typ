#import "metadata/register.typ": register
#import "metadata/schema.typ": schema

// Pass the configured function to note observation; realize it with metadata().
#let zk_metadata = register.with(schema: schema)
