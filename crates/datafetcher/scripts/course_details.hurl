GET https://enr-apps.andrew.cmu.edu/open/SOC/SOCServlet/courseDetails?COURSE={{course}}&SEMESTER={{season}}{{year}}

HTTP/1.1 200

[Captures]
schedule: xpath "/html/body/div/div[1]/div/table/tbody"
urls: xpath "/html/body/div/div[2]/div[1]/dl/dd/ul"

special_permission: xpath "string(/html/body/div/div[2]/div[2]/dl/dd)"
description: xpath "string(/html/body/div/div[3]/div/p)"
prereqs: xpath "string(/html/body/div/div[4]/div[1]/dl/dd)"
coreqs: xpath "string(/html/body/div/div[4]/div[2]/dl/dd)"
crosslisted: xpath "string(/html/body/div/div[5]/div[1]/dl/dd)"
notes: xpath "string(/html/body/div/div[5]/div[2]/dl/dd)"

reservations: xpath "/html/body/div/div[6]/div/table/tbody"
