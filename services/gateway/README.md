# Gateway Service

The gateway service is responsible for communicating with Discord's WebSocket Gateway

Events received are validated to ensure they are in the correct packet format as per the [Discord Developer Docs](https://discord.com/developers/docs/topics/gateway)

Depending on the packet the gateway will select a worker to send the packet to from Redis. If will then send the packet over a Redis pub/sub channel which the corresponding worker should be listening on.

The worker is expected to respond to allow the gateway to know the packet is being handled correctly. If this does not happen the gateway will remove that worker from Redis and select another.

If for whatever reason no workers are avaible, the gateway may call the Discord API to response to a user.
