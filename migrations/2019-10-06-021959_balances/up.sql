CREATE VIEW balances AS
SELECT users.name,
       SUM(transactions.amount) balance
FROM transactions,
     users
WHERE users.id == transactions.user
GROUP BY user;
