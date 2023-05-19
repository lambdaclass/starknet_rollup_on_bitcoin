defmodule BridgeWeb.PageController do
  use BridgeWeb, :controller

  def home(conn, _params) do
    # The home page is often custom made,
    # so skip the default app layout.
    render(conn, :home, layout: false)
  end

  # POST action
  # "msg" should be the tx_id
  def burn(conn, params) do
    case Curvy.verify(params["sig"], params["msg"], params["key"]) do
      true -> redirect(conn, to: Routes.live_path(conn, BridgeWeb.TransactionLive,  msg: params["msg"]))
      false -> conn
              |> put_status(:bad_request)
              |> json(%{error: "Signature could not be verified"})
              |> halt()
    end

    send_resp(conn, 200, "Success")
  end


  defp call_ord_decoder(args) do
    {port, _} = Port.open({:spawn_executable, "/target/debug/ord-decoder"}, [:binary, args: args, exit_status: true])

    receive do
      {^port, {:exit_status, 0}} ->
        :ok

      {^port, {:exit_status, status}} ->
        {:error, "Ordinal decoder exited with status #{status}"}

      {^port, {:data, data}} ->
        IO.puts("Ordinal decoder was succesful and returned: #{data}")
    end
  end

end
