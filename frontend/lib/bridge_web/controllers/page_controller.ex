defmodule BridgeWeb.PageController do
  use BridgeWeb, :controller

  def home(conn, _params) do
    # The home page is often custom made,
    # so skip the default app layout.
    render(conn, :home, layout: false)
  end

  #POST action
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

end
