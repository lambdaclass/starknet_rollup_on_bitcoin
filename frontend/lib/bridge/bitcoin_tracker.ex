defmodule BitcoinTracker do
  use GenServer
  require Logger
  alias HTTPoison

  @blockcypher_url "https://api.blockcypher.com/v1/btc/main/txs/"
  @poll_interval 20_000 # 20 seconds in milliseconds
  @timeout 20 * 60 * 1_000 # 20 minutes in milliseconds

  def start_link(live_view_pid, tx_id) do
    GenServer.start_link(__MODULE__, {live_view_pid, tx_id}, name: __MODULE__)
  end

  def init({live_view_pid, tx_id}) do
    Process.send_after(self(), :poll, 0)
    {:ok, {tx_id, 0, live_view_pid}}
  end

  def handle_info(:poll, {tx_id, elapsed_time, live_view_pid}) do
    if elapsed_time < @timeout do
      case get_transaction(tx_id) do
        {:ok, true, tx} ->
          Logger.info("Transaction #{tx_id} confirmed!")
          Phoenix.PubSub.broadcast(Bridge.PubSub, "transaction_status", {:transaction_status, tx_id, "Confirmed"})
          token_params = %{
            "tx_id" => tx_id,
            "tx" => tx
          }

          #Insert into DB
          case Bridge.Tokens.create_token(token_params) do
            {:ok, _token} ->
              IO.puts("Token created successfully.")

            #{:error, %Ecto.Changeset{} = changeset} ->
          end

          {:stop, :normal, nil}

          {:ok, false}  ->
            Logger.info("Transaction #{tx_id} still not confirmed")
            Phoenix.PubSub.broadcast(Bridge.PubSub, "transaction_status", {:transaction_status, tx_id, "Pending confirmation"})
            Process.send_after(self(), :poll, @poll_interval)
            {:noreply, {tx_id, elapsed_time + @poll_interval, live_view_pid}}

        _ ->
          Process.send_after(self(), :poll, @poll_interval)
          {:noreply, {tx_id, elapsed_time + @poll_interval, live_view_pid}}
      end
    else
      Logger.error("Timeout: Transaction not confirmed after 20 minutes.")
      Phoenix.PubSub.broadcast(Bridge.PubSub, "transaction_status", {:transaction_status, tx_id, "timeout"})
      {:stop, :normal, nil}
    end
  end

  defp get_transaction(tx_id) do
    transaction_url = "#{@blockcypher_url}#{tx_id}"

    case HTTPoison.get(transaction_url) do
      {:ok, %HTTPoison.Response{status_code: 200, body: body}} ->
        body
        |> Jason.decode!()
        |> Map.fetch("confirmed")
        |> case do
          {:ok, false} -> {:ok, false}
          {:ok, _} -> {:ok, true, body}
          :error -> {:ok, false}
        end

      _ -> {:error, :request_failed}
    end
  end
end
