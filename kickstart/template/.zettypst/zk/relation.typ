// Dependency edges must form a DAG with one color per ordered endpoint pair.
// Incoming relations derive lifecycle metadata while preserving source origins.
#import "relation/element.typ": colored-edge, colors, evolves-from, replaces
#import "relation/observe.typ": outgoing, references, relations
#import "relation/validate.typ": dependency-colors, validate
#import "relation/graph.typ": dag, derive, evaluate
