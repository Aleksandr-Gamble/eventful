# eventful 

In a microservice architecture, it is the responsibility of microservices to emit events. This module is intended to both
1) Make it more ergonomic to produce and cosume events.
2) Abstract the act of producing (and consuming) events to allow a level of agnosticism regarding which particular message bus / event queueing service you select.

