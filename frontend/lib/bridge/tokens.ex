defmodule Bridge.Tokens do
  alias Bridge.Repo
  alias Bridge.Tokens.Token

  def create_token(attrs \\ %{}) do
    %Token{}
    |> Token.changeset(attrs)
    |> Repo.insert()
  end
end
