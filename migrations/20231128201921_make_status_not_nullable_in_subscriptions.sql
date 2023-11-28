BEGIN;
  update subscriptions set status = 'confirmed' where status is null;
  ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
