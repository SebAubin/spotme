use saphir::*;
use saphir::Method;
use serde_urlencoded::from_str;
use std::net::SocketAddr;
use std::str::FromStr;
use crate::models::RepositoryCollection;
use crate::models::Repository;
use crate::models::ip::Ip;
use mongodb::coll::options::FindOptions;

pub struct LookupController {
    dispatch: ControllerDispatch<RepositoryCollection>,
}

impl LookupController {
    pub fn new(repos: RepositoryCollection) -> Self {
        let dispatch = ControllerDispatch::new(repos);
        dispatch.add(Method::GET,
                     reg!(r"^ip-lookup$"),
                     ip_lookup);

        LookupController {
            dispatch
        }
    }
}

impl Controller for LookupController {
    fn handle(&self, req: &mut SyncRequest, res: &mut SyncResponse) {
        self.dispatch.dispatch(req, res);
    }

    fn base_path(&self) -> &str {
        "^/"
    }
}

fn parse(repos: &RepositoryCollection, req: &SyncRequest, res: &mut SyncResponse) {
    /*if let Ok(ips) = repos.ip.get_all() {
        for mut ip in ips {
            let network_vec = ip.network.split(".").collect::<Vec<&str>>();
            let net = network_vec[0];
            let sub = network_vec[1];
            let sub2 = network_vec[2];
            let netmask = network_vec[3];
            ip.net = Some(net.parse::<i32>().unwrap());
            ip.sub = Some(sub.parse::<i32>().unwrap());
            ip.sub2 = Some(sub2.parse::<i32>().unwrap());
            ip.netmask = Some(netmask.to_string());

            if let Ok(_) = repos.ip.delete_by_id(ip.id.clone()) {
                //println!("Deleted!");
            }

            repos.ip.insert(ip.clone());

            /**/
        }
    } else {
        println!("Nope");
    }*/
}

fn ip_lookup(repos: &RepositoryCollection, req: &SyncRequest, res: &mut SyncResponse) {
    res.status(StatusCode::BAD_REQUEST);

    let error_json = json!({
        "error": "The request IP was not found in the database."
    });

    if let Some(query) = req.uri().query() {
        if let Ok(params) = serde_urlencoded::from_str::<Vec<(String, String)>>(query) {
            if let Some(ip) = params.get(0) {
                if ip.0 == "ip" {
                    match SocketAddr::from_str(&format!("{}:80", &ip.1)) {
                        Ok(addr) => {
                            let ip_str = addr.ip().to_string();
                            let ip_parts = ip_str.split(".").collect::<Vec<&str>>();
                            let net: i32 = ip_parts[0].parse::<i32>().expect("Ok");
                            let sub: i32 = ip_parts[1].parse::<i32>().expect("Ok");
                            let sub2: i32 = ip_parts[2].parse::<i32>().expect("Ok");
                            let mask: i32 = ip_parts[3].parse::<i32>().expect("Ok");

                            let filtered_ips = match repos.ip.count(Some(doc! {"net":net, "sub":sub})) {
                                Ok(count) if count > 0 => {
                                    let ips_sub2 = match repos.ip.count(Some(doc! {"net":net, "sub":sub, "sub2": sub2})) {
                                        Ok(count) if count > 0 => {
                                            repos.ip.find(doc! {"net":net, "sub":sub, "sub2": sub2}).unwrap()
                                        }
                                        _ => {
                                            let mut options = FindOptions::new();
                                            options.limit = Some(sub2 as i64);
                                            options.sort = Some(doc!{"sub": -1});
                                            let mut lte_ips_sub = repos.ip.find_with_options(doc! {"net":net, "sub": sub, "sub2": { "$lte": sub2 }}, options).unwrap();

                                            if let Some(ref ip) = lte_ips_sub.first() {
                                                let l_sub = ip.sub2;
                                                while let Some(_) = lte_ips_sub.iter().position(|i| i.sub2 != l_sub).map(|idx| lte_ips_sub.remove(idx)) {}
                                            }

                                            lte_ips_sub
                                        }
                                    };

                                    ips_sub2
                                }
                                _ => {
                                    let mut options = FindOptions::new();
                                    options.limit = Some(sub2 as i64);
                                    options.sort = Some(doc!{"sub": -1});
                                    let mut lte_ips_sub = repos.ip.find_with_options(doc! {"net":net, "sub": { "$lte": sub }, "sub2": { "$lte": sub2 }}, options).unwrap();

                                    if let Some(ref ip) = lte_ips_sub.first() {
                                        let l_sub = ip.sub;
                                        while let Some(_) = lte_ips_sub.iter().position(|i| i.sub != l_sub).map(|idx| lte_ips_sub.remove(idx)) {}
                                    }

                                    if lte_ips_sub.len() < 1 {
                                        res.status(StatusCode::OK);
                                        res.body(serde_json::to_string(&error_json).expect("Will be ok"));
                                    }

                                    lte_ips_sub
                                }
                            };

                            if filtered_ips.len() > 0{
                                let the_right_one = find_right_entry(&filtered_ips);

                                if let Ok(Some(location)) = repos.location.get(doc! {"geoname_id": the_right_one.geoname_id}) {
                                    let json = json!({
                                    "request_ip": ip_str,
                                    "network": the_right_one.network,
                                    "lat": the_right_one.latitude,
                                    "lon": the_right_one.longitude,
                                    "accuracy": the_right_one.accuracy_radius,
                                    "continent": location.continent_name,
                                    "country": location.country_name,
                                    "subdivision_1_name": location.subdivision_1_name,
                                    "subdivision_2_name": location.subdivision_2_name,
                                    "city_name": location.city_name,
                                    "time_zone": location.time_zone,
                                });

                                    res.status(StatusCode::OK);
                                    res.body(serde_json::to_string(&json).expect("Will be ok"));
                                }
                            }
                            else{
                                res.status(StatusCode::OK);
                                res.body(serde_json::to_string(&error_json).expect("Will be ok"));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn find_right_entry(ips: &Vec<Ip>) -> Ip {
    let mut vec_value = get_netmask_array(ips);
    vec_value.sort_by(|a, b| a.0.cmp(&b.0));
    vec_value.last().unwrap().1.clone()
}

fn get_netmask_array(ips: &Vec<Ip>) -> Vec<(i64, &Ip)> {
    let mut vec_ips = Vec::new();
    for ip in ips {
        let val = get_netmask_value(ip.netmask.as_str());
        vec_ips.push((val, ip));
    }

    vec_ips
}

fn get_netmask_value(netmask: &str) -> i64 {
    let cyrille = netmask.split('/').collect::<Vec<_>>();

    (cyrille[0].parse::<i64>().unwrap() * 8) + (cyrille[1].parse::<i64>().unwrap() * 256)
}