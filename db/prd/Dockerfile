FROM postgres:12-alpine

ADD ./create-db.sh /docker-entrypoint-initdb.d/
ADD ./entrypoint.sh /app/

ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["postgres"]
