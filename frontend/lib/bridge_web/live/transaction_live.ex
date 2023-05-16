defmodule BridgeWeb.TransactionLive do
  use Phoenix.LiveView

  def render(assigns) do
    #BridgeWeb.TransactionView.render("index.html", assigns)
    ~H"""
    Transaction status: <%= @status %>
    """
  end

  def mount(%{"tx_id" => tx_id}, _session, socket) do
    if connected?(socket) do
      Phoenix.PubSub.subscribe(Bridge.PubSub, "transaction_status")
      {:ok, _pid} = BitcoinTracker.start_link(self(), tx_id)
    end

    {:ok, assign(socket, :status, "Waiting for transaction status...")}
  end

  def handle_info({:transaction_status, _tx_id, status}, socket) do
    {:noreply, assign(socket, status: status)}
  end
end
