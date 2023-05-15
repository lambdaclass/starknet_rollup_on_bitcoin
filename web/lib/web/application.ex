defmodule Web.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      # Start the Telemetry supervisor
      WebWeb.Telemetry,
      # Start the PubSub system
      {Phoenix.PubSub, name: Web.PubSub},
      # Start Finch
      {Finch, name: Web.Finch},
      # Start the Endpoint (http/https)
      WebWeb.Endpoint
      # Start a worker by calling: Web.Worker.start_link(arg)
      # {Web.Worker, arg}
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Web.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    WebWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
