defmodule Bridge.Repo.Migrations.AddTable do
  use Ecto.Migration

  def change do
    create table(:tokens) do
      add :tx_id, :string
      add :tx, :string
    end
  end
end
