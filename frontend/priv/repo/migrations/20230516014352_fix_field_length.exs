defmodule Bridge.Repo.Migrations.FixFieldLength do
  use Ecto.Migration

  def change do
        alter table(:tokens) do
          modify :tx, :string, size: 5120
    end
  end
end
