class UserService
  def initialize(database)
    @db = database
  end

  def find_user(id)
    @db.query("SELECT * FROM users WHERE id = ?", id)
  end

  def create_user(name, email)
    @db.query("INSERT INTO users (name, email) VALUES (?, ?)", name, email)
  end

  def update_user(id, attrs)
    user = find_user(id)
    user.update(attrs)
    user.save
  end

  def delete_user(id)
    @db.query("DELETE FROM users WHERE id = ?", id)
  end

  def list_users
    @db.query("SELECT * FROM users")
  end

  def count_users
    @db.query("SELECT COUNT(*) FROM users").first
  end
end
