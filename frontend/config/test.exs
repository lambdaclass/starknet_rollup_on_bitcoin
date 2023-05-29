import Config

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :web, BridgeWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  secret_key_base: "fjvyI53AkAk9Kq1REX3BjciqnYylaHh/uj4ngTMdBytgl8zGEmF1cj/6tCZBGQMY",
  server: false

# In test we don't send emails.
config :web, Bridge.Mailer,
  adapter: Swoosh.Adapters.Test

# Disable swoosh api client as it is only required for production adapters.
config :swoosh, :api_client, false

# Print only warnings and errors during test
config :logger, level: :warning

# Initialize plugs at runtime for faster test compilation
config :phoenix, :plug_init_mode, :runtime
