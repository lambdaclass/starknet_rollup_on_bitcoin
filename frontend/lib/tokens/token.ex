defmodule Bridge.Tokens.Token do
  use Ecto.Schema
  import Ecto.Changeset

  schema "tokens" do
    field :tx_id, :string
    field :tx, :string
  end

  def changeset(token, attrs) do
    token
    |> cast(attrs, [:tx_id, :tx])
    |> validate_required([:tx_id, :tx])
  end
end
