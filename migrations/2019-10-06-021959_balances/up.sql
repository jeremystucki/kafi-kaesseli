CREATE VIEW balances AS
SELECT users.id user_id,
       users.name,
       SUM(transactions.amount) amount
FROM transactions,
     users
WHERE users.id == transactions.user
GROUP BY user_id;
