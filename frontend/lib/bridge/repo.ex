defmodule Bridge.Repo do
  use Ecto.Repo,
    otp_app: :web,
    adapter: Ecto.Adapters.Postgres
end
